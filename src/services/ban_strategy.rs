use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, warn};

use crate::{
    claude_web_state::ClaudeWebState,
    config::CLEWDR_CONFIG,
    error::ClewdrError,
    types::claude::{CreateMessageParams, Message, RequiredMessageParams, Role},
};

/// Types of errors that can occur during ban operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorType {
    /// Rate limiting error (HTTP 429)
    RateLimit,
    /// Account has been banned (HTTP 403/401)
    Banned,
    /// Network connectivity issues
    NetworkError,
    /// Server-side errors (HTTP 5xx)
    ServerError,
    /// Unknown or uncategorized errors
    Unknown,
}

/// Metrics and performance data for ban strategy optimization
#[derive(Debug, Clone, Serialize)]
pub struct StrategyMetrics {
    /// Total number of requests made
    pub total_requests: u64,
    /// Number of successful requests
    pub successful_requests: u64,
    /// Number of failed requests
    pub failed_requests: u64,
    /// Average response time for successful requests
    pub average_response_time: Duration,
    /// The last error type encountered
    pub last_error: Option<ErrorType>,
    /// When the last request was made (as timestamp since Unix epoch)
    pub last_request_timestamp: Option<u64>,
    /// Number of consecutive errors (used for adaptive throttling)
    pub consecutive_errors: u32,
    /// Current adaptive delay calculated based on performance
    pub adaptive_delay: Duration,
}

impl Default for StrategyMetrics {
    fn default() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            average_response_time: Duration::from_millis(1000),
            last_error: None,
            last_request_timestamp: None,
            consecutive_errors: 0,
            adaptive_delay: Duration::from_millis(1000),
        }
    }
}

/// Pool of User-Agent strings for fingerprint randomization
///
/// This pool contains a variety of realistic browser User-Agent strings
/// to help avoid detection by simulating different browsers and platforms.
#[derive(Debug, Clone)]
pub struct UserAgentPool {
    agents: Vec<String>,
}

impl UserAgentPool {
    pub fn new() -> Self {
        let agents = vec![
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string(),
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string(),
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string(),
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/121.0".to_string(),
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:109.0) Gecko/20100101 Firefox/121.0".to_string(),
            "Mozilla/5.0 (X11; Linux x86_64; rv:109.0) Gecko/20100101 Firefox/121.0".to_string(),
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.2.1 Safari/605.1.15".to_string(),
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 Edg/120.0.0.0".to_string(),
        ];
        Self { agents }
    }

    pub fn random(&self) -> String {
        let mut rng = rand::thread_rng();
        self.agents.choose(&mut rng).cloned().unwrap_or_else(|| {
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string()
        })
    }
}

pub struct BanStrategy {
    metrics: Arc<RwLock<HashMap<String, StrategyMetrics>>>,
    user_agents: UserAgentPool,
    proxy_pool: Vec<String>,
}

impl BanStrategy {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            user_agents: UserAgentPool::new(),
            proxy_pool: Vec::new(),
        }
    }

    pub async fn execute_request(
        &self,
        cookie: &str,
        prompt: &str,
        model: &str,
    ) -> Result<wreq::Response, ClewdrError> {
        let config = CLEWDR_CONFIG.load();

        // Check working hours
        if !config.ban.is_within_working_hours() {
            warn!("Outside working hours, skipping request");
            return Err(ClewdrError::BadRequest {
                msg: "Outside working hours",
            });
        }

        let mut state = ClaudeWebState::new_minimal();
        state.set_cookie(cookie)?;

        // Apply random User-Agent if enabled
        if config.ban.user_agent_rotation {
            self.apply_random_user_agent(&mut state).await?;
        }

        // Apply proxy rotation if enabled
        if config.ban.proxy_rotation && !self.proxy_pool.is_empty() {
            self.apply_random_proxy(&mut state).await?;
        }

        // Calculate adaptive delay
        let delay = self.calculate_adaptive_delay(cookie).await;
        if delay > Duration::ZERO {
            tokio::time::sleep(delay).await;
        }

        // Create request parameters
        let params = CreateMessageParams::new(RequiredMessageParams {
            model: model.to_string(),
            messages: vec![Message::new_text(Role::User, prompt.to_string())],
            max_tokens: config.ban.max_tokens(),
        })
        .with_stream(false);

        // Execute request with timing
        let start_time = Instant::now();
        let result = state.send_raw(params).await;
        let response_time = start_time.elapsed();

        // Update metrics - extract string info before passing
        let result_str = if let Ok(_) = result {
            "success".to_string()
        } else if let Err(e) = &result {
            e.to_string()
        } else {
            "unknown".to_string()
        };

        self.update_metrics_with_string(cookie, result_str, response_time)
            .await;

        result
    }

    async fn apply_random_user_agent(&self, state: &mut ClaudeWebState) -> Result<(), ClewdrError> {
        let user_agent = self.user_agents.random();
        debug!("Applying User-Agent: {}", user_agent);
        state.set_user_agent(user_agent);
        Ok(())
    }

    async fn apply_random_proxy(&self, state: &mut ClaudeWebState) -> Result<(), ClewdrError> {
        if self.proxy_pool.is_empty() {
            return Ok(());
        }

        let mut rng = rand::thread_rng();
        let proxy =
            self.proxy_pool
                .choose(&mut rng)
                .ok_or_else(|| ClewdrError::UnexpectedNone {
                    msg: "No proxy available in proxy pool",
                })?;
        debug!("Applying proxy: {}", proxy);
        if let Ok(parsed) = wreq::Proxy::all(proxy) {
            state.set_proxy(parsed)?;
        }
        Ok(())
    }

    pub async fn calculate_adaptive_delay(&self, cookie: &str) -> Duration {
        let config = CLEWDR_CONFIG.load();
        if !config.ban.adaptive_throttling {
            return config.ban.get_jitter_delay();
        }

        let metrics = self.metrics.read().await;
        if let Some(metric) = metrics.get(cookie) {
            let base_delay = config.ban.get_jitter_delay();

            // Adjust delay based on consecutive errors
            if metric.consecutive_errors > 0 {
                let multiplier = metric.consecutive_errors as u64;
                Duration::from_millis(base_delay.as_millis() as u64 * multiplier)
            } else if metric.average_response_time > Duration::from_secs(10) {
                // Slow responses mean we should slow down
                Duration::from_millis(base_delay.as_millis() as u64 * 3)
            } else {
                base_delay
            }
        } else {
            config.ban.get_jitter_delay()
        }
    }

    async fn update_metrics_with_string(
        &self,
        cookie: &str,
        result_str: String,
        response_time: Duration,
    ) {
        // Extract all needed info before await to avoid Send issues
        let is_success = result_str == "success";
        let error_info = if !is_success {
            let error_str = result_str.to_lowercase();
            Some(
                if error_str.contains("rate limit") || error_str.contains("429") {
                    ErrorType::RateLimit
                } else if error_str.contains("banned")
                    || error_str.contains("403")
                    || error_str.contains("401")
                {
                    ErrorType::Banned
                } else if error_str.contains("network") || error_str.contains("connection") {
                    ErrorType::NetworkError
                } else if error_str.contains("server") || error_str.contains("500") {
                    ErrorType::ServerError
                } else {
                    ErrorType::Unknown
                },
            )
        } else {
            None
        };

        let mut metrics = self.metrics.write().await;
        let metric = metrics.entry(cookie.to_string()).or_default();

        metric.total_requests += 1;
        metric.last_request_timestamp = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        );

        if is_success {
            metric.successful_requests += 1;
            metric.consecutive_errors = 0;
            metric.last_error = None;

            // Update average response time
            let total_time = metric.average_response_time * (metric.successful_requests - 1) as u32
                + response_time;
            metric.average_response_time = total_time / metric.successful_requests as u32;
        } else {
            metric.failed_requests += 1;
            metric.consecutive_errors += 1;
            metric.last_error = error_info;
        }

        // Update adaptive delay
        let config = CLEWDR_CONFIG.load();
        if config.ban.adaptive_throttling {
            let base_delay = config.ban.get_jitter_delay();
            if metric.consecutive_errors > 0 {
                metric.adaptive_delay = Duration::from_millis(
                    base_delay.as_millis() as u64 * metric.consecutive_errors as u64,
                );
            } else if metric.average_response_time > Duration::from_secs(5) {
                metric.adaptive_delay = Duration::from_millis(base_delay.as_millis() as u64 * 2);
            } else {
                metric.adaptive_delay = base_delay;
            }
        }
    }

    pub async fn should_retry(&self, cookie: &str, error_str: String) -> bool {
        let config = CLEWDR_CONFIG.load();
        if !config.ban.smart_error_handling {
            return true;
        }

        // Lowercase error string for comparison
        let error_str = error_str.to_lowercase();

        let metrics = self.metrics.read().await;
        if let Some(metric) = metrics.get(cookie) {
            if metric.consecutive_errors >= config.ban.retry_attempts {
                return false;
            }

            // Don't retry banned errors
            if error_str.contains("banned")
                || error_str.contains("403")
                || error_str.contains("401")
            {
                return false;
            }
        }

        true
    }

    pub async fn get_metrics(&self, cookie: &str) -> Option<StrategyMetrics> {
        self.metrics.read().await.get(cookie).cloned()
    }

    pub async fn get_all_metrics(&self) -> HashMap<String, StrategyMetrics> {
        self.metrics.read().await.clone()
    }

    pub async fn clear_metrics(&self, cookie: &str) {
        let mut metrics = self.metrics.write().await;
        metrics.remove(cookie);
    }

    pub async fn clear_all_metrics(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.clear();
    }
}

impl Default for BanStrategy {
    fn default() -> Self {
        Self::new()
    }
}
