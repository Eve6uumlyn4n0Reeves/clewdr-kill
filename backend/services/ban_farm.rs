use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};

use chrono::{DateTime, Utc};

use tokio::{
    sync::{Mutex, Notify, RwLock},
    task::JoinHandle,
    time::{sleep, timeout},
};
use tracing::{error, info, warn};

use crate::{
    config::{BanConfig, CLEWDR_CONFIG},
    error::ClewdrError,
    services::{
        ban_queue::BanQueueHandle,
        ban_strategy::{BanStrategy, StrategyExecutor},
        prompt_loader::PromptLoader,
    },
};

pub struct BanFarm {
    queue: BanQueueHandle,
    prompts: RwLock<Arc<PromptLoader>>,
    config: RwLock<BanConfig>,
    strategy: Arc<dyn StrategyExecutor>,
    /// 使用原子变量存储退避结束时间戳(纳秒),避免锁竞争
    backoff_until_nanos: AtomicU64,
    /// 用于通知worker退避结束
    backoff_notify: Arc<Notify>,
    mode: RwLock<OperationMode>,
    worker_handles: Mutex<Vec<JoinHandle<()>>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationMode {
    Running,
    Paused,
    Stopped,
}

impl BanFarm {
    pub async fn spawn(queue: BanQueueHandle) -> Result<Arc<Self>, ClewdrError> {
        let config = CLEWDR_CONFIG.load().ban.clone();
        let prompts = Arc::new(PromptLoader::load(&config.prompts_dir).map_err(|e| {
            error!(
                "Prompt loader failed for directory {}: {}",
                config.prompts_dir, e
            );
            e
        })?);

        let strategy: Arc<dyn StrategyExecutor> = Arc::new(BanStrategy::new());
        let farm = Arc::new(Self {
            queue,
            prompts: RwLock::new(prompts),
            config: RwLock::new(config),
            strategy,
            backoff_until_nanos: AtomicU64::new(0),
            backoff_notify: Arc::new(Notify::new()),
            mode: RwLock::new(OperationMode::Running),
            worker_handles: Mutex::new(Vec::new()),
        });
        // 测试环境可通过环境变量禁用 worker，避免对外网请求
        if std::env::var("CLEWDR_DISABLE_WORKERS").is_ok() {
            let mut mode = farm.mode.write().await;
            *mode = OperationMode::Paused;
            tracing::info!("Workers disabled by CLEWDR_DISABLE_WORKERS");
        } else if farm.prompts.read().await.len() > 0 {
            let concurrency = farm.config.read().await.concurrency.max(1);
            farm.launch_workers(concurrency).await;
        } else {
            let mut mode = farm.mode.write().await;
            *mode = OperationMode::Paused;
            tracing::warn!(
                "No prompts found in {}. Workers paused until prompts are added.",
                config.prompts_dir
            );
        }
        Ok(farm)
    }

    pub async fn strategy_metrics(
        &self,
    ) -> std::collections::HashMap<String, crate::services::ban_strategy::StrategyMetrics> {
        self.strategy.get_all_metrics().await
    }

    pub async fn reset_strategy_metrics(&self) {
        self.strategy.clear_all_metrics().await;
    }

    pub async fn reload_config(self: &Arc<Self>, new_config: BanConfig) -> Result<(), ClewdrError> {
        let current_config = self.config.read().await.clone();

        // 始终重载提示词，以感知新增/删除
        self.reload_prompts_internal(&new_config).await?;

        {
            let mut config_guard = self.config.write().await;
            *config_guard = new_config.clone();
        }

        if new_config.concurrency != current_config.concurrency {
            if std::env::var("CLEWDR_DISABLE_WORKERS").is_ok() {
                let mut mode = self.mode.write().await;
                *mode = OperationMode::Paused;
                warn!(
                    "BanFarm concurrency changed ({} -> {}), but workers are disabled by CLEWDR_DISABLE_WORKERS; restart required to apply",
                    current_config.concurrency,
                    new_config.concurrency
                );
            } else {
                self.restart_workers(new_config.concurrency).await?;
                info!(
                    "Worker pool restarted with concurrency {} -> {}",
                    current_config.concurrency, new_config.concurrency
                );
            }
        }

        Ok(())
    }

    pub async fn worker_count(&self) -> usize {
        self.worker_handles.lock().await.len()
    }

    pub async fn prompt_count(&self) -> usize {
        self.prompts.read().await.len()
    }

    pub async fn sample_prompt(&self) -> Option<String> {
        self.prompts.read().await.random_prompt()
    }

    pub async fn current_mode_for_test(&self) -> OperationMode {
        *self.mode.read().await
    }

    pub async fn pause(&self) {
        let mut mode = self.mode.write().await;
        *mode = OperationMode::Paused;
    }

    pub async fn resume(&self) {
        let mut mode = self.mode.write().await;
        *mode = OperationMode::Running;
    }

    pub async fn stop(&self) {
        let mut mode = self.mode.write().await;
        *mode = OperationMode::Stopped;
    }

    async fn current_mode(&self) -> OperationMode {
        *self.mode.read().await
    }

    async fn launch_workers(self: &Arc<Self>, concurrency: usize) {
        let prompt_count = self.prompts.read().await.len();
        info!(
            "Ban farm starting with {} worker(s) and {} prompt(s)",
            concurrency, prompt_count
        );
        let mut handles = self.worker_handles.lock().await;
        for idx in 0..concurrency {
            let farm = Arc::clone(self);
            let handle = tokio::spawn(async move {
                farm.worker_loop(idx).await;
            });
            handles.push(handle);
        }
    }

    async fn stop_workers(&self) -> Result<(), ClewdrError> {
        {
            let mut mode = self.mode.write().await;
            *mode = OperationMode::Stopped;
        }

        let mut handles = self.worker_handles.lock().await;
        let mut join_handles = std::mem::take(&mut *handles);
        drop(handles);

        for handle in join_handles.drain(..) {
            match timeout(Duration::from_secs(10), handle).await {
                Ok(Ok(())) => {}
                Ok(Err(e)) => warn!("Worker task panicked: {}", e),
                Err(_) => warn!("Timed out waiting for worker shutdown"),
            }
        }

        Ok(())
    }

    async fn restart_workers(self: &Arc<Self>, new_concurrency: usize) -> Result<(), ClewdrError> {
        info!("Restarting workers with concurrency {}", new_concurrency);
        self.stop_workers().await?;

        {
            let mut mode = self.mode.write().await;
            *mode = OperationMode::Running;
        }

        let target = new_concurrency.max(1);
        self.launch_workers(target).await;
        Ok(())
    }

    /// 内部：在给定配置下重载提示词并视情况启动/暂停 worker
    async fn reload_prompts_internal(&self, cfg: &BanConfig) -> Result<usize, ClewdrError> {
        let prompts = Arc::new(PromptLoader::load(&cfg.prompts_dir)?);
        let count = prompts.len();

        {
            let mut guard = self.prompts.write().await;
            *guard = prompts;
        }

        if count == 0 {
            // 关闭现有 worker，但保持模式为 Paused 以便后续自动恢复
            {
                let mut handles = self.worker_handles.lock().await;
                let mut join_handles = std::mem::take(&mut *handles);
                drop(handles);
                for handle in join_handles.drain(..) {
                    if let Err(e) = timeout(Duration::from_secs(5), handle).await {
                        warn!("Timeout stopping worker after prompts cleared: {}", e);
                    }
                }
            }
            {
                let mut mode = self.mode.write().await;
                *mode = OperationMode::Paused;
            }
            tracing::warn!(
                "No prompts available after reload ({}). Workers paused.",
                cfg.prompts_dir
            );
            return Ok(0);
        }

        // 若当前没有 worker（启动时缺少提示词），则按现有并发启动
        if self.worker_handles.lock().await.is_empty() {
            let concurrency = cfg.concurrency.max(1);
            self.launch_workers(concurrency).await;
            let mut mode = self.mode.write().await;
            *mode = OperationMode::Running;
        }

        tracing::info!(
            "Prompts reloaded from {} ({} file(s))",
            cfg.prompts_dir,
            count
        );

        Ok(count)
    }

    /// 重新加载提示词（不改变其他配置），返回加载到的数量
    pub async fn reload_prompts(&self) -> Result<usize, ClewdrError> {
        let cfg = self.config.read().await.clone();
        self.reload_prompts_internal(&cfg).await
    }

    async fn worker_loop(self: Arc<Self>, worker_id: usize) {
        let mut consecutive_failures: u32 = 0;
        let max_consecutive_failures: u32 = 5;
        loop {
            match self.current_mode().await {
                OperationMode::Stopped => {
                    info!("Worker {}: Stopped", worker_id);
                    break;
                }
                OperationMode::Paused => {
                    sleep(Duration::from_millis(500)).await;
                    continue;
                }
                OperationMode::Running => {}
            }

            // Check for global backoff (e.g. rate limiting / overload)
            if let Some(delay) = self.current_backoff_delay().await {
                info!(
                    "Worker {}: Global backoff active for {:?}",
                    worker_id, delay
                );
                sleep(delay).await;
                continue;
            }

            // Get a cookie from queue
            let cookie = match self.queue.pop().await {
                Ok(c) => c,
                Err(ClewdrError::NoCookieAvailable) => {
                    info!("Worker {}: No cookies available, waiting", worker_id);
                    sleep(Duration::from_secs(5)).await;
                    continue;
                }
                Err(e) => {
                    error!("Worker {}: Queue error: {}", worker_id, e);
                    sleep(Duration::from_secs(5)).await;
                    continue;
                }
            };

            let config_snapshot = self.config.read().await.clone();

            // Get random prompt
            let Some(prompt) = self.prompts.read().await.random_prompt() else {
                warn!("Worker {}: no prompts available", worker_id);
                sleep(Duration::from_secs(30)).await;
                continue;
            };

            // Get first available model (prefer Haiku for cost efficiency)
            let model = config_snapshot
                .models
                .first()
                .cloned()
                .unwrap_or_else(|| "claude-3-5-haiku-20241022".to_string());

            // Age-aware aggressiveness
            let age_hours = cookie
                .submitted_at
                .as_deref()
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| {
                    Utc::now()
                        .signed_duration_since(dt.with_timezone(&Utc))
                        .num_hours()
                })
                .unwrap_or(0);

            // Use advanced strategy for request execution
            let cookie_str = cookie.cookie.to_string();
            match self
                .strategy
                .execute_request(&cookie_str, &prompt, &model)
                .await
            {
                Ok(_) => {
                    consecutive_failures = 0;
                    info!(
                        "Worker {}: Request sent successfully ({})",
                        worker_id,
                        cookie.cookie.ellipse()
                    );
                    if let Err(e) = self
                        .queue
                        .mark_processed_with_retry(cookie_str.clone(), false, None, None, None, 3)
                        .await
                    {
                        warn!(
                            "Worker {}: mark_processed failed after success ({}): {}",
                            worker_id,
                            cookie.cookie.ellipse(),
                            e
                        );
                    }
                }
                Err(e) => {
                    consecutive_failures = consecutive_failures.saturating_add(1);
                    let err_str = e.to_string().to_lowercase();
                    let now = chrono::Utc::now();

                    if err_str.contains("banned")
                        || err_str.contains("403")
                        || err_str.contains("401")
                    {
                        info!(
                            "Worker {}: Cookie banned ({})",
                            worker_id,
                            cookie.cookie.ellipse()
                        );
                        if let Err(e) = self
                            .queue
                            .mark_processed_with_retry(
                                cookie_str.clone(),
                                true,
                                Some("banned".to_string()),
                                None,
                                None,
                                3,
                            )
                            .await
                        {
                            warn!(
                                "Worker {}: mark_processed failed for banned cookie ({}): {}",
                                worker_id,
                                cookie.cookie.ellipse(),
                                e
                            );
                        }
                    } else if err_str.contains("rate limit") || err_str.contains("429") {
                        warn!(
                            "Worker {}: Rate limited ({}), pausing",
                            worker_id,
                            cookie.cookie.ellipse()
                        );
                        // 越接近 48h，冷却越短
                        let cooldown_minutes = if age_hours >= 40 {
                            10
                        } else if age_hours >= 24 {
                            20
                        } else {
                            30
                        };
                        if let Err(e) = self
                            .queue
                            .mark_processed_with_retry(
                                cookie_str.clone(),
                                false,
                                Some("rate_limited".to_string()),
                                Some(now + chrono::Duration::minutes(cooldown_minutes)),
                                Some(now),
                                3,
                            )
                            .await
                        {
                            warn!(
                                "Worker {}: mark_processed failed for rate limited cookie ({}): {}",
                                worker_id,
                                cookie.cookie.ellipse(),
                                e
                            );
                        }
                        self.set_global_backoff().await;
                    } else {
                        warn!(
                            "Worker {}: Request failed ({}): {}",
                            worker_id,
                            cookie.cookie.ellipse(),
                            e
                        );
                        if let Err(e) = self
                            .queue
                            .mark_processed_with_retry(
                                cookie_str.clone(),
                                false,
                                Some(err_str),
                                None,
                                None,
                                3,
                            )
                            .await
                        {
                            warn!(
                                "Worker {}: mark_processed failed for error case ({}): {}",
                                worker_id,
                                cookie.cookie.ellipse(),
                                e
                            );
                        }
                    }

                    if consecutive_failures >= max_consecutive_failures {
                        let delay = Duration::from_secs(60 * consecutive_failures as u64);
                        warn!(
                            "Worker {}: {} consecutive failures, backing off for {:?}",
                            worker_id, consecutive_failures, delay
                        );
                        sleep(delay).await;
                    }
                }
            }

            // Delay between requests respects configuration
            let mut delay_seconds = config_snapshot.pause_seconds.max(1);
            // 越接近 48h，加快重放节奏
            if age_hours >= 40 {
                delay_seconds = delay_seconds.saturating_div(3).max(2);
            } else if age_hours >= 24 {
                delay_seconds = delay_seconds.saturating_div(2).max(5);
            }
            sleep(Duration::from_secs(delay_seconds)).await;
        }
    }
}

impl BanFarm {
    /// 检查当前是否在退避期内
    /// 使用原子操作,无锁竞争
    async fn current_backoff_delay(&self) -> Option<Duration> {
        let until_nanos = self.backoff_until_nanos.load(Ordering::Acquire);
        if until_nanos == 0 {
            return None;
        }

        let now = std::time::SystemTime::now();
        let now_nanos = now
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;

        if now_nanos < until_nanos {
            let remaining_nanos = until_nanos.saturating_sub(now_nanos);
            Some(Duration::from_nanos(remaining_nanos))
        } else {
            // 退避已结束,重置为0
            self.backoff_until_nanos.store(0, Ordering::Relaxed);
            None
        }
    }

    /// 设置全局退避
    /// 使用原子变量+Notify,避免锁竞争和死锁风险
    async fn set_global_backoff(&self) {
        // 读取配置(仅需读锁,快速释放)
        let pause_seconds = {
            let config_guard = self.config.read().await;
            config_guard.pause_seconds.max(1)
        };

        // 计算退避结束时间戳(纳秒)
        let now = std::time::SystemTime::now();
        let now_nanos = now
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        let until_nanos = now_nanos + (pause_seconds * 1_000_000_000);

        // 原子操作设置时间戳,无锁
        self.backoff_until_nanos
            .store(until_nanos, Ordering::Release);

        info!("Global backoff set for {} seconds", pause_seconds);
    }
}
