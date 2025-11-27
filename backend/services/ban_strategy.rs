use std::{collections::HashMap, sync::Arc, time::Duration, time::Instant};

use async_trait::async_trait;
use serde::Serialize;
use tokio::sync::RwLock;

use crate::{
    claude_web_state::ClaudeWebState,
    config::CLEWDR_CONFIG,
    error::ClewdrError,
    types::claude::{CreateMessageParams, Message, RequiredMessageParams, Role},
};

#[async_trait]
pub trait StrategyExecutor: Send + Sync {
    async fn execute_request(
        &self,
        cookie: &str,
        prompt: &str,
        model: &str,
    ) -> Result<wreq::Response, ClewdrError>;

    async fn get_all_metrics(&self) -> HashMap<String, StrategyMetrics>;

    async fn clear_all_metrics(&self);
}

/// Simple metrics for tracking ban operations
#[derive(Debug, Clone, Serialize)]
pub struct StrategyMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time: Duration,
    pub last_error: Option<String>,
}

impl Default for StrategyMetrics {
    fn default() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            average_response_time: Duration::from_millis(1000),
            last_error: None,
        }
    }
}

pub struct BanStrategy {
    metrics: Arc<RwLock<HashMap<String, StrategyMetrics>>>,
}

impl BanStrategy {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn update_metrics(
        &self,
        cookie: &str,
        is_success: bool,
        response_time: Duration,
        last_error: Option<String>,
    ) {
        let mut metrics = self.metrics.write().await;
        let metric = metrics.entry(cookie.to_string()).or_default();

        metric.total_requests += 1;

        if is_success {
            metric.successful_requests += 1;
            // Update average response time
            let total_time = metric.average_response_time * (metric.successful_requests - 1) as u32
                + response_time;
            metric.average_response_time = total_time / metric.successful_requests as u32;
            metric.last_error = None;
        } else {
            metric.failed_requests += 1;
            metric.last_error = last_error;
        }
    }

    pub async fn get_all_metrics(&self) -> std::collections::HashMap<String, StrategyMetrics> {
        self.metrics.read().await.clone()
    }

    pub async fn clear_all_metrics(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.clear();
    }
}

#[async_trait]
impl StrategyExecutor for BanStrategy {
    async fn execute_request(
        &self,
        cookie: &str,
        prompt: &str,
        model: &str,
    ) -> Result<wreq::Response, ClewdrError> {
        let config = CLEWDR_CONFIG.load();

        let mut state = ClaudeWebState::new_minimal();
        state.set_cookie(cookie)?;

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

        let last_error = result.as_ref().err().map(|e| e.to_string());
        let is_success = result.is_ok();
        self.update_metrics(cookie, is_success, response_time, last_error)
            .await;

        result
    }

    async fn get_all_metrics(&self) -> HashMap<String, StrategyMetrics> {
        self.metrics.read().await.clone()
    }

    async fn clear_all_metrics(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.clear();
    }
}

impl Default for BanStrategy {
    fn default() -> Self {
        Self::new()
    }
}
