use axum::{Json, extract::State};
use axum_auth::AuthBearer;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::LazyLock, time::Instant};
use sysinfo::{CpuRefreshKind, RefreshKind, System};

use super::error::ApiError;
use super::stats_history;
use crate::{config::CLEWDR_CONFIG, router::AppState};

static START_TIME: LazyLock<Instant> = LazyLock::new(Instant::now);

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemStats {
    pub total_cookies: u64,
    pub pending_cookies: u64,
    pub banned_cookies: u64,
    pub total_requests: u64,
    pub requests_per_minute: f64,
    pub success_rate: f64,
    pub average_response_time: u64, // milliseconds
    pub workers_active: u32,
    pub uptime_seconds: u64,
    pub last_update: DateTime<Utc>,
    pub error_distribution: HashMap<String, u64>,
    pub performance_metrics: PerformanceMetrics,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub cpu_usage: f64,
    pub memory_usage: u64,           // bytes
    pub network_latency: u64,        // milliseconds
    pub queue_processing_time: u64,  // milliseconds
    pub strategy_effectiveness: f64, // percentage
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CookieMetrics {
    pub cookie_id: String,
    pub requests_sent: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time: u64,
    pub last_request_time: Option<DateTime<Utc>>,
    pub consecutive_errors: u32,
    pub adaptive_delay: u64,
    pub status: String, // "pending", "banned", "active"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HistoricalStats {
    pub timestamps: Vec<DateTime<Utc>>,
    pub request_counts: Vec<u64>,
    pub success_rates: Vec<f64>,
    pub response_times: Vec<u64>,
    pub error_rates: Vec<f64>,
}

/// Get system-wide statistics
pub async fn api_get_system_stats(
    State(app_state): State<AppState>,
    AuthBearer(token): AuthBearer,
) -> Result<Json<SystemStats>, ApiError> {
    if !CLEWDR_CONFIG.load().admin_auth(&token) {
        return Err(ApiError::unauthorized());
    }

    let queue_handle = app_state.ban_queue_handle;
    let queue_info = queue_handle.get_status().await.map_err(|e| {
        tracing::error!("Failed to get queue status: {}", e);
        ApiError::internal("Failed to get queue status")
    })?;

    // Strategy metrics（若可用）
    let strategy_metrics = app_state.ban_farm.strategy_metrics().await;
    let (strategy_total, strategy_success, avg_resp_ms, error_distribution) = {
        let mut total = 0u64;
        let mut success = 0u64;
        let mut resp_acc: u128 = 0;
        let mut resp_count: u64 = 0;
        let mut err_dist: HashMap<String, u64> = HashMap::new();
        for metric in strategy_metrics.values() {
            total += metric.total_requests;
            success += metric.successful_requests;
            if metric.average_response_time.as_millis() > 0 {
                resp_acc += metric.average_response_time.as_millis();
                resp_count += 1;
            }
            if let Some(err) = &metric.last_error {
                let key = format!("{err:?}");
                *err_dist.entry(key).or_default() += 1;
            }
        }
        let avg_resp = if resp_count > 0 {
            (resp_acc / resp_count as u128) as u64
        } else {
            0
        };
        (total, success, avg_resp, err_dist)
    };

    // Calculate statistics
    let total_cookies = (queue_info.pending.len() + queue_info.banned.len()) as u64;
    let pending_cookies = queue_info.pending.len() as u64;
    let banned_cookies = queue_info.banned.len() as u64;
    let total_requests = queue_info.total_requests.max(strategy_total);

    // Calculate requests per minute using uptime
    let uptime_seconds = get_uptime_seconds().max(1);
    let requests_per_minute = total_requests as f64 / (uptime_seconds as f64 / 60.0);

    // Calculate success rate
    let success_rate = if total_requests > 0 {
        if strategy_total > 0 {
            (strategy_success as f64 / strategy_total as f64) * 100.0
        } else {
            (total_requests.saturating_sub(banned_cookies) as f64 / total_requests as f64) * 100.0
        }
    } else {
        0.0
    };

    // Get performance metrics (derived)
    let performance_metrics = get_performance_metrics(&queue_info, avg_resp_ms, success_rate);

    let stats = SystemStats {
        total_cookies,
        pending_cookies,
        banned_cookies,
        total_requests,
        requests_per_minute,
        success_rate,
        average_response_time: if avg_resp_ms > 0 {
            avg_resp_ms
        } else {
            performance_metrics.network_latency
        },
        workers_active: app_state.ban_farm.worker_count() as u32,
        uptime_seconds,
        last_update: Utc::now(),
        error_distribution: if !error_distribution.is_empty() {
            error_distribution
        } else {
            HashMap::new()
        },
        performance_metrics,
    };

    // 记录历史样本
    stats_history::record_sample(super::stats_history::StatsSample {
        timestamp: stats.last_update,
        total_requests: stats.total_requests,
        success_rate: stats.success_rate,
        average_response_time: stats.average_response_time,
    });

    Ok(Json(stats))
}

/// Get detailed metrics for specific cookies
pub async fn api_get_cookie_metrics(
    State(app_state): State<AppState>,
    AuthBearer(token): AuthBearer,
) -> Result<Json<Vec<CookieMetrics>>, ApiError> {
    if !CLEWDR_CONFIG.load().admin_auth(&token) {
        return Err(ApiError::unauthorized());
    }

    let queue_handle = app_state.ban_queue_handle;
    let queue_info = queue_handle.get_status().await.map_err(|e| {
        tracing::error!("Failed to get queue status: {}", e);
        ApiError::internal("Failed to get queue status")
    })?;

    let mut cookie_metrics = Vec::new();

    // Process pending cookies
    for cookie in queue_info.pending {
        let metrics = CookieMetrics {
            cookie_id: cookie.cookie.to_string(),
            requests_sent: cookie.requests_sent,
            successful_requests: cookie.requests_sent.saturating_sub(1), // Simplified
            failed_requests: 0,                                          // Simplified
            average_response_time: 1500,                                 // Simplified
            last_request_time: cookie.last_used_at.and_then(|s| {
                DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|dt| dt.with_timezone(&Utc))
            }),
            consecutive_errors: 0,
            adaptive_delay: 500,
            status: "pending".to_string(),
        };
        cookie_metrics.push(metrics);
    }

    // Process banned cookies
    for cookie in queue_info.banned {
        let metrics = CookieMetrics {
            cookie_id: cookie.cookie.to_string(),
            requests_sent: cookie.requests_sent,
            successful_requests: cookie.requests_sent.saturating_sub(1), // Simplified
            failed_requests: 1,          // At least one failed request
            average_response_time: 2000, // Simplified
            last_request_time: cookie.last_used_at.and_then(|s| {
                DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|dt| dt.with_timezone(&Utc))
            }),
            consecutive_errors: 1,
            adaptive_delay: 1000,
            status: "banned".to_string(),
        };
        cookie_metrics.push(metrics);
    }

    Ok(Json(cookie_metrics))
}

/// Get historical statistics
pub async fn api_get_historical_stats(
    State(_app_state): State<AppState>,
    AuthBearer(token): AuthBearer,
    Json(_params): Json<HistoricalStatsParams>,
) -> Result<Json<HistoricalStats>, ApiError> {
    if !CLEWDR_CONFIG.load().admin_auth(&token) {
        return Err(ApiError::unauthorized());
    }

    // 返回单点快照，避免伪造数据
    let samples = stats_history::get_samples();
    if samples.is_empty() {
        let now = Utc::now();
        return Ok(Json(HistoricalStats {
            timestamps: vec![now],
            request_counts: vec![0],
            success_rates: vec![0.0],
            response_times: vec![0],
            error_rates: vec![0.0],
        }));
    }

    let mut timestamps = Vec::with_capacity(samples.len());
    let mut request_counts = Vec::with_capacity(samples.len());
    let mut success_rates = Vec::with_capacity(samples.len());
    let mut response_times = Vec::with_capacity(samples.len());
    let mut error_rates = Vec::with_capacity(samples.len());

    for s in samples {
        timestamps.push(s.timestamp);
        request_counts.push(s.total_requests);
        success_rates.push(s.success_rate);
        response_times.push(s.average_response_time);
        error_rates.push(100.0 - s.success_rate);
    }

    Ok(Json(HistoricalStats {
        timestamps,
        request_counts,
        success_rates,
        response_times,
        error_rates,
    }))
}

#[derive(Debug, Deserialize)]
pub struct HistoricalStatsParams {
    pub interval_minutes: i64,
    pub points: Option<usize>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}

/// Reset statistics
pub async fn api_reset_stats(
    State(app_state): State<AppState>,
    AuthBearer(token): AuthBearer,
) -> Result<(), ApiError> {
    if !CLEWDR_CONFIG.load().admin_auth(&token) {
        return Err(ApiError::unauthorized());
    }

    app_state
        .ban_queue_handle
        .reset_stats()
        .await
        .map_err(|e| {
            tracing::error!("Failed to reset queue stats: {}", e);
            ApiError::internal("Failed to reset queue stats")
        })?;

    app_state.ban_farm.reset_strategy_metrics().await;

    Ok(())
}

// Helper functions
fn get_performance_metrics(
    queue_info: &crate::services::ban_queue::BanQueueInfo,
    avg_resp_ms: u64,
    success_rate: f64,
) -> PerformanceMetrics {
    let sys = System::new_with_specifics(
        RefreshKind::new()
            .with_cpu(CpuRefreshKind::new().with_cpu_usage())
            .with_memory(sysinfo::MemoryRefreshKind::new()),
    );
    let cpu_usage = sys.global_cpu_info().cpu_usage() as f64;
    let memory_usage = sys.used_memory() * 1024; // convert KiB to bytes

    PerformanceMetrics {
        cpu_usage,
        memory_usage,
        network_latency: avg_resp_ms,
        queue_processing_time: (queue_info.pending.len() as u64) * 10,
        strategy_effectiveness: success_rate,
    }
}

fn get_uptime_seconds() -> u64 {
    START_TIME.elapsed().as_secs()
}
