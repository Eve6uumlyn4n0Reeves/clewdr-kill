use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;
use tracing::{debug, warn};

/// Simple in-memory rate limiter
#[derive(Debug, Clone)]
pub struct RateLimiter {
    state: Arc<RwLock<RateLimiterState>>,
}

#[derive(Debug)]
struct RateLimiterState {
    // Map IP address to request timestamps
    requests: HashMap<String, Vec<Instant>>,
    // Maximum requests per window
    max_requests: usize,
    // Time window in seconds
    window_seconds: u64,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window_seconds: u64) -> Self {
        Self {
            state: Arc::new(RwLock::new(RateLimiterState {
                requests: HashMap::new(),
                max_requests,
                window_seconds,
            })),
        }
    }

    /// Check if a request from the given IP should be allowed
    pub async fn is_allowed(&self, ip: &str) -> bool {
        let mut state = self.state.write().await;
        // max_requests == 0 表示不做限流（自用默认）
        if state.max_requests == 0 {
            state.requests.clear();
            return true;
        }
        let now = Instant::now();
        let window_start = now - Duration::from_secs(state.window_seconds);
        let max_requests = state.max_requests;

        // Get or create request history for this IP
        let requests = state
            .requests
            .entry(ip.to_string())
            .or_insert_with(Vec::new);

        // Remove old requests outside the window
        requests.retain(|&timestamp| timestamp > window_start);

        // Check if under the limit
        if requests.len() < max_requests {
            requests.push(now);
            true
        } else {
            warn!("Rate limit exceeded for IP: {}", ip);
            false
        }
    }

    /// Clean up old entries to prevent memory leaks
    pub async fn cleanup(&self) {
        let mut state = self.state.write().await;
        if state.max_requests == 0 {
            state.requests.clear();
            return;
        }
        let now = Instant::now();
        let window_start = now - Duration::from_secs(state.window_seconds);

        state.requests.retain(|_, requests| {
            requests.retain(|&timestamp| timestamp > window_start);
            !requests.is_empty()
        });

        debug!(
            "Rate limiter cleanup completed, tracking {} IPs",
            state.requests.len()
        );
    }
}

// 默认不做限流（自用场景），若要启用可自行实例化带上限的 RateLimiter
pub fn default_rate_limiter() -> RateLimiter {
    RateLimiter::new(0, 60)
}
