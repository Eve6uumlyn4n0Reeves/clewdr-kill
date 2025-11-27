use axum::{extract::State, Json};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::LazyLock,
    time::{Duration, Instant},
};
use sysinfo::{CpuRefreshKind, RefreshKind, System};
use tokio::sync::RwLock;

use super::stats_history;
use crate::{
    api::response::{success_response, ApiResponse, EmptyResponse},
    config::CLEWDR_CONFIG,
    db::{NewStats, Queries},
    error::ClewdrError,
    router::AppState,
    services::ban_queue::BanQueueInfo,
};

static START_TIME: LazyLock<Instant> = LazyLock::new(Instant::now);
static SYSTEM_STATS_CACHE: LazyLock<RwLock<SystemStatsCache>> =
    LazyLock::new(|| RwLock::new(SystemStatsCache::new()));

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
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
) -> Result<Json<ApiResponse<SystemStats>>, ClewdrError> {
    let stats = cached_system_stats(&app_state).await?;
    Ok(Json(ApiResponse::success(stats)))
}

/// Get detailed metrics for specific cookies
pub async fn api_get_cookie_metrics(
    State(app_state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<CookieMetrics>>>, ClewdrError> {
    let queue_handle = app_state.ban_queue_handle.clone();
    let queue_info = queue_handle.get_status().await.map_err(|e| {
        tracing::error!("Failed to get queue status: {}", e);
        ClewdrError::Whatever {
            message: "Failed to get queue status".to_string(),
            source: Some(Box::new(e)),
        }
    })?;

    let mut cookie_metrics = Vec::new();
    let strategy_metrics = app_state.ban_farm.strategy_metrics().await;
    let pause_delay_ms = CLEWDR_CONFIG.load().ban.pause_seconds.saturating_mul(1000);

    // Process pending cookies
    for cookie in queue_info.pending {
        let metrics_entry = strategy_metrics.get(&cookie.cookie.to_string()).cloned();
        let (successful_requests, failed_requests, avg_resp) = metrics_entry
            .map(|m| {
                (
                    m.successful_requests,
                    m.failed_requests,
                    m.average_response_time.as_millis() as u64,
                )
            })
            .unwrap_or_else(|| (cookie.requests_sent, 0, 0));

        let metrics = CookieMetrics {
            cookie_id: cookie.cookie.to_string(),
            requests_sent: cookie.requests_sent,
            successful_requests,
            failed_requests,
            average_response_time: avg_resp,
            last_request_time: cookie.last_used_at.and_then(|s| {
                DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|dt| dt.with_timezone(&Utc))
            }),
            consecutive_errors: failed_requests.min(u32::MAX as u64) as u32,
            adaptive_delay: pause_delay_ms,
            status: "pending".to_string(),
        };
        cookie_metrics.push(metrics);
    }

    // Process processing cookies
    for cookie in queue_info.processing {
        let metrics_entry = strategy_metrics.get(&cookie.cookie.to_string()).cloned();
        let (successful_requests, failed_requests, avg_resp) = metrics_entry
            .map(|m| {
                (
                    m.successful_requests,
                    m.failed_requests,
                    m.average_response_time.as_millis() as u64,
                )
            })
            .unwrap_or_else(|| (cookie.requests_sent, 0, 0));

        let metrics = CookieMetrics {
            cookie_id: cookie.cookie.to_string(),
            requests_sent: cookie.requests_sent,
            successful_requests,
            failed_requests,
            average_response_time: avg_resp,
            last_request_time: cookie.last_used_at.and_then(|s| {
                DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|dt| dt.with_timezone(&Utc))
            }),
            consecutive_errors: failed_requests.min(u32::MAX as u64) as u32,
            adaptive_delay: pause_delay_ms,
            status: "processing".to_string(),
        };
        cookie_metrics.push(metrics);
    }

    // Process banned cookies
    for cookie in queue_info.banned {
        let metrics_entry = strategy_metrics.get(&cookie.cookie.to_string()).cloned();
        let (successful_requests, failed_requests, avg_resp) = metrics_entry
            .map(|m| {
                (
                    m.successful_requests,
                    m.failed_requests.max(1),
                    m.average_response_time.as_millis() as u64,
                )
            })
            .unwrap_or_else(|| (cookie.requests_sent.saturating_sub(1), 1, 0));

        let metrics = CookieMetrics {
            cookie_id: cookie.cookie.to_string(),
            requests_sent: cookie.requests_sent,
            successful_requests,
            failed_requests,
            average_response_time: avg_resp,
            last_request_time: cookie.last_used_at.and_then(|s| {
                DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|dt| dt.with_timezone(&Utc))
            }),
            consecutive_errors: failed_requests.min(u32::MAX as u64) as u32,
            adaptive_delay: pause_delay_ms,
            status: "banned".to_string(),
        };
        cookie_metrics.push(metrics);
    }

    Ok(Json(ApiResponse::success(cookie_metrics)))
}

/// Get historical statistics
pub async fn api_get_historical_stats(
    State(app_state): State<AppState>,
    Json(params): Json<HistoricalStatsParams>,
) -> Result<Json<ApiResponse<HistoricalStats>>, ClewdrError> {
    let history = fetch_historical_stats(&app_state, &params).await?;
    Ok(Json(ApiResponse::success(history)))
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
) -> Result<Json<EmptyResponse>, ClewdrError> {
    app_state
        .ban_queue_handle
        .reset_stats()
        .await
        .map_err(|e| {
            tracing::error!("Failed to reset queue stats: {}", e);
            ClewdrError::Whatever {
                message: "Failed to reset queue stats".to_string(),
                source: Some(Box::new(e)),
            }
        })?;

    app_state.ban_farm.reset_strategy_metrics().await;
    invalidate_stats_cache().await;

    Ok(Json(success_response()))
}

fn get_uptime_seconds() -> u64 {
    START_TIME.elapsed().as_secs()
}

#[derive(Clone)]
struct StatsPersistence {
    total_requests: u64,
    success_count: u64,
    error_count: u64,
    average_response_time: u64,
}

#[derive(Clone)]
struct CachedStats {
    stats: SystemStats,
    persistence: StatsPersistence,
    captured_at: Instant,
}

struct SystemStatsCache {
    cached: Option<CachedStats>,
    last_persisted_total: u64,
    last_persisted_at: Option<Instant>,
}

impl SystemStatsCache {
    fn new() -> Self {
        Self {
            cached: None,
            last_persisted_total: 0,
            last_persisted_at: None,
        }
    }

    async fn get(&mut self, app_state: &AppState) -> Result<SystemStats, ClewdrError> {
        let needs_refresh = self
            .cached
            .as_ref()
            .map(|c| c.captured_at.elapsed() >= Duration::from_secs(5))
            .unwrap_or(true);

        if needs_refresh {
            let snapshot = collect_system_stats(app_state).await?;
            let stats_clone = snapshot.stats.clone();
            self.cached = Some(snapshot);
            self.persist_if_needed(app_state).await?;
            return Ok(stats_clone);
        }

        Ok(self
            .cached
            .as_ref()
            .expect("cache checked above")
            .stats
            .clone())
    }

    async fn persist_if_needed(&mut self, app_state: &AppState) -> Result<(), ClewdrError> {
        let Some(cached) = &self.cached else {
            return Ok(());
        };
        if cached.persistence.total_requests <= self.last_persisted_total {
            return Ok(());
        }
        let should_persist = self
            .last_persisted_at
            .map(|t| t.elapsed() >= Duration::from_secs(60))
            .unwrap_or(true);
        if !should_persist {
            return Ok(());
        }

        persist_stats_sample(app_state, &cached.persistence).await?;
        self.last_persisted_total = cached.persistence.total_requests;
        self.last_persisted_at = Some(Instant::now());
        Ok(())
    }

    fn invalidate(&mut self) {
        self.cached = None;
        self.last_persisted_total = 0;
        self.last_persisted_at = None;
    }
}

async fn cached_system_stats(app_state: &AppState) -> Result<SystemStats, ClewdrError> {
    let mut guard = SYSTEM_STATS_CACHE.write().await;
    guard.get(app_state).await
}

async fn invalidate_stats_cache() {
    let mut guard = SYSTEM_STATS_CACHE.write().await;
    guard.invalidate();
}

async fn collect_system_stats(app_state: &AppState) -> Result<CachedStats, ClewdrError> {
    let aggregated = Queries::get_aggregated_stats(app_state.db.pool())
        .await
        .map_err(|e| {
            tracing::error!("Failed to get aggregated stats: {}", e);
            e
        })?;
    tracing::info!(
        total_cookies = aggregated.total_cookies,
        pending = aggregated.pending_count,
        banned = aggregated.banned_count,
        total_requests = aggregated.total_requests,
        "Aggregated stats snapshot"
    );

    let queue_handle = app_state.ban_queue_handle.clone();
    let queue_info = queue_handle.get_status().await.map_err(|e| {
        tracing::error!("Failed to get queue status: {}", e);
        ClewdrError::Whatever {
            message: "Failed to get queue status".to_string(),
            source: Some(Box::new(e)),
        }
    })?;

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

    let total_cookies = aggregated.total_cookies.max(0) as u64;
    let pending_cookies = aggregated.pending_count.max(0) as u64;
    let banned_cookies = aggregated.banned_count.max(0) as u64;
    let total_requests = aggregated.total_requests.max(strategy_total as i64) as u64;
    let uptime_seconds = get_uptime_seconds().max(1);
    let requests_per_minute = total_requests as f64 / (uptime_seconds as f64 / 60.0);

    let success_count = if strategy_total > 0 {
        strategy_success
    } else if total_requests > 0 {
        total_requests.saturating_sub(banned_cookies)
    } else {
        0
    };
    let error_count = total_requests.saturating_sub(success_count);

    let success_rate = if total_requests > 0 {
        (success_count as f64 / total_requests as f64) * 100.0
    } else {
        0.0
    };

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
        workers_active: app_state.ban_farm.worker_count().await as u32,
        uptime_seconds,
        last_update: Utc::now(),
        error_distribution: if !error_distribution.is_empty() {
            error_distribution
        } else {
            HashMap::new()
        },
        performance_metrics,
    };

    stats_history::record_sample(super::stats_history::StatsSample {
        timestamp: stats.last_update,
        total_requests: stats.total_requests,
        success_rate: stats.success_rate,
        average_response_time: stats.average_response_time,
    });

    Ok(CachedStats {
        stats,
        persistence: StatsPersistence {
            total_requests,
            success_count,
            error_count,
            average_response_time: avg_resp_ms,
        },
        captured_at: Instant::now(),
    })
}

fn get_performance_metrics(
    queue_info: &BanQueueInfo,
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

async fn persist_stats_sample(
    app_state: &AppState,
    sample: &StatsPersistence,
) -> Result<(), ClewdrError> {
    let new_stats = NewStats {
        total_requests: sample.total_requests.min(i64::MAX as u64) as i64,
        success_count: sample.success_count.min(i64::MAX as u64) as i64,
        error_count: sample.error_count.min(i64::MAX as u64) as i64,
        avg_response_time: sample.average_response_time as f64,
    };
    Queries::create_stats(app_state.db.pool(), new_stats).await?;
    Ok(())
}

async fn fetch_historical_stats(
    app_state: &AppState,
    params: &HistoricalStatsParams,
) -> Result<HistoricalStats, ClewdrError> {
    let rows = select_stats_rows(app_state, params).await?;
    if rows.is_empty() {
        // Fallback to in-memory samples for short-lived进程
        let samples = stats_history::get_samples();
        if samples.is_empty() {
            let now = Utc::now();
            return Ok(HistoricalStats {
                timestamps: vec![now],
                request_counts: vec![0],
                success_rates: vec![0.0],
                response_times: vec![0],
                error_rates: vec![0.0],
            });
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

        return Ok(HistoricalStats {
            timestamps,
            request_counts,
            success_rates,
            response_times,
            error_rates,
        });
    }

    let mut hist = HistoricalStats {
        timestamps: Vec::with_capacity(rows.len()),
        request_counts: Vec::with_capacity(rows.len()),
        success_rates: Vec::with_capacity(rows.len()),
        response_times: Vec::with_capacity(rows.len()),
        error_rates: Vec::with_capacity(rows.len()),
    };

    for row in rows {
        let total = row.total_requests.max(0) as u64;
        let success = row.success_count.max(0) as u64;
        let error = row.error_count.max(0) as u64;
        let success_rate = if total > 0 {
            (success as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        let error_rate = if total > 0 {
            (error as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        hist.timestamps.push(row.timestamp);
        hist.request_counts.push(total);
        hist.success_rates.push(success_rate);
        hist.response_times
            .push(row.avg_response_time.max(0.0) as u64);
        hist.error_rates.push(error_rate);
    }

    Ok(hist)
}

async fn select_stats_rows(
    app_state: &AppState,
    params: &HistoricalStatsParams,
) -> Result<Vec<crate::db::Stats>, ClewdrError> {
    let pool = app_state.db.pool();
    if params.start_time.is_some() || params.end_time.is_some() {
        let end = params.end_time.unwrap_or_else(Utc::now);
        let interval_minutes = params.interval_minutes.max(1);
        let start = params
            .start_time
            .unwrap_or_else(|| end - chrono::Duration::minutes(interval_minutes));

        Queries::get_stats_between(pool, start, end).await
    } else {
        let limit = params.points.unwrap_or(24).clamp(1, 288) as i64;
        let mut rows = Queries::get_recent_stats(pool, limit).await?;
        rows.reverse();
        Ok(rows)
    }
}
