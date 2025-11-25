use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use rand::seq::SliceRandom;
use tokio::{sync::RwLock, time::sleep};
use tracing::{error, info, warn};

use crate::{
    config::{BanConfig, CLEWDR_CONFIG},
    error::ClewdrError,
    services::{ban_queue::BanQueueHandle, ban_strategy::BanStrategy, prompt_loader::PromptLoader},
};

pub struct BanFarm {
    queue: BanQueueHandle,
    prompts: Arc<PromptLoader>,
    config: BanConfig,
    strategy: Arc<BanStrategy>,
    backoff_until: RwLock<Option<Instant>>,
    mode: RwLock<OperationMode>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
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
        if prompts.is_empty() {
            return Err(ClewdrError::PathNotFound {
                msg: format!(
                    "Prompt directory {} did not contain any usable prompts",
                    config.prompts_dir
                ),
            });
        }

        let strategy = Arc::new(BanStrategy::new());
        let farm = Arc::new(Self {
            queue,
            prompts,
            config,
            strategy,
            backoff_until: RwLock::new(None),
            mode: RwLock::new(OperationMode::Running),
        });
        farm.launch_workers();
        Ok(farm)
    }

    pub async fn strategy_metrics(
        &self,
    ) -> std::collections::HashMap<String, crate::services::ban_strategy::StrategyMetrics>
    {
        self.strategy.get_all_metrics().await
    }

    pub async fn reset_strategy_metrics(&self) {
        self.strategy.clear_all_metrics().await;
    }

    pub fn worker_count(&self) -> usize {
        self.config.concurrency.max(1)
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

    fn launch_workers(self: &Arc<Self>) {
        let concurrency = self.config.concurrency.max(1);
        info!(
            "Ban farm starting with {} worker(s) and {} prompt(s)",
            concurrency,
            self.prompts.len()
        );
        for idx in 0..concurrency {
            let farm = Arc::clone(self);
            tokio::spawn(async move {
                farm.worker_loop(idx).await;
            });
        }
    }

    async fn worker_loop(self: Arc<Self>, worker_id: usize) {
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

            // Get random prompt
            let Some(prompt) = self.prompts.random_prompt() else {
                warn!("Worker {}: no prompts available", worker_id);
                sleep(Duration::from_secs(30)).await;
                continue;
            };

            // Get random model - create rng here to avoid Send issues
            let model = {
                let mut rng = rand::thread_rng();
                self.config.models.choose(&mut rng).cloned()
            };

            let Some(model) = model else {
                warn!("Worker {}: no models configured", worker_id);
                sleep(Duration::from_secs(60)).await;
                continue;
            };

            // Use advanced strategy for request execution
            let cookie_str = cookie.cookie.to_string();
            match self
                .strategy
                .execute_request(&cookie_str, &prompt, &model)
                .await
            {
                Ok(_) => {
                    info!(
                        "Worker {}: Request sent successfully ({})",
                        worker_id,
                        cookie.cookie.ellipse()
                    );
                }
                Err(e) => {
                    let err_str = e.to_string();
                    let banned_like = err_str.contains("banned")
                        || err_str.contains("disabled")
                        || err_str.contains("403")
                        || err_str.contains("401");

                    let lower = err_str.to_lowercase();
                    let rate_limited = lower.contains("rate limit")
                        || lower.contains("too many requests")
                        || lower.contains("429");

                    if banned_like {
                        info!(
                            "Worker {}: Cookie likely banned ({})",
                            worker_id,
                            cookie.cookie.ellipse()
                        );
                        let _ = self.queue.mark_banned(cookie.cookie.to_string()).await;
                    } else if rate_limited {
                        warn!(
                            "Worker {}: Rate limit or overload detected ({}): {}. Global pause {}s",
                            worker_id,
                            cookie.cookie.ellipse(),
                            err_str,
                            self.config.pause_seconds
                        );
                        self.set_global_backoff().await;
                    } else {
                        // Check if we should retry using smart error handling
                        if self.config.smart_error_handling {
                            let error_str = e.to_string();
                            if !self.strategy.should_retry(&cookie_str, error_str).await {
                                warn!(
                                    "Worker {}: Max retries exceeded for cookie ({}): {}",
                                    worker_id,
                                    cookie.cookie.ellipse(),
                                    e
                                );
                                continue;
                            }
                        }

                        // Apply adaptive delay instead of fixed backoff
                        let delay = self.strategy.calculate_adaptive_delay(&cookie_str).await;
                        warn!(
                            "Worker {}: Request failed ({}): {}. Backing off for {:?}",
                            worker_id,
                            cookie.cookie.ellipse(),
                            e,
                            delay
                        );
                        sleep(delay).await;
                    }
                }
            }

            // Small delay between requests based on jitter配置
            sleep(self.config.get_jitter_delay()).await;
        }
    }
}

impl BanFarm {
    async fn current_backoff_delay(&self) -> Option<Duration> {
        let guard = self.backoff_until.read().await;
        if let Some(until) = *guard {
            let now = Instant::now();
            if now < until {
                return Some(until.saturating_duration_since(now));
            }
        }
        None
    }

    async fn set_global_backoff(&self) {
        let pause = Duration::from_secs(self.config.pause_seconds.max(1));
        let mut guard = self.backoff_until.write().await;
        let until = Instant::now() + pause;
        *guard = Some(until);
    }
}
