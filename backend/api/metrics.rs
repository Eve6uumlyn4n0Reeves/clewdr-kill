use axum::{extract::State, response::IntoResponse};
use http::{header, HeaderValue, StatusCode};

use crate::{
    db::Queries,
    error::ClewdrError,
    router::AppState,
};

/// Prometheus 风格的文本指标输出
pub async fn api_metrics(State(app_state): State<AppState>) -> Result<impl IntoResponse, ClewdrError> {
    // 队列信息
    let queue = app_state.ban_queue_handle.get_status().await.map_err(|e| {
        tracing::error!("Failed to get queue status for metrics: {}", e);
        ClewdrError::Whatever {
            message: "metrics queue status".into(),
            source: Some(Box::new(e)),
        }
    })?;

    // 聚合统计
    let agg = Queries::get_aggregated_stats(app_state.db.pool()).await.map_err(|e| {
        tracing::error!("Failed to get aggregated stats for metrics: {}", e);
        ClewdrError::Whatever {
            message: "metrics aggregated stats".into(),
            source: Some(Box::new(e)),
        }
    })?;

    let workers = app_state.ban_farm.worker_count().await as u64;

    let mut body = String::new();
    // Gauges
    body.push_str("# HELP clewdr_queue_pending Pending cookies\n");
    body.push_str("# TYPE clewdr_queue_pending gauge\n");
    body.push_str(&format!("clewdr_queue_pending {}\n", queue.pending.len()));

    body.push_str("# HELP clewdr_queue_processing Processing cookies\n");
    body.push_str("# TYPE clewdr_queue_processing gauge\n");
    body.push_str(&format!("clewdr_queue_processing {}\n", queue.processing.len()));

    body.push_str("# HELP clewdr_queue_banned Banned cookies\n");
    body.push_str("# TYPE clewdr_queue_banned gauge\n");
    body.push_str(&format!("clewdr_queue_banned {}\n", queue.banned.len()));

    body.push_str("# HELP clewdr_queue_total_requests Total requests counted by queue\n");
    body.push_str("# TYPE clewdr_queue_total_requests counter\n");
    body.push_str(&format!("clewdr_queue_total_requests {}\n", queue.total_requests));

    body.push_str("# HELP clewdr_workers_active Active workers\n");
    body.push_str("# TYPE clewdr_workers_active gauge\n");
    body.push_str(&format!("clewdr_workers_active {}\n", workers));

    body.push_str("# HELP clewdr_cookies_total Total cookies stored\n");
    body.push_str("# TYPE clewdr_cookies_total gauge\n");
    body.push_str(&format!("clewdr_cookies_total {}\n", agg.total_cookies));

    body.push_str("# HELP clewdr_cookies_pending_total Pending cookies stored\n");
    body.push_str("# TYPE clewdr_cookies_pending_total gauge\n");
    body.push_str(&format!("clewdr_cookies_pending_total {}\n", agg.pending_count));

    body.push_str("# HELP clewdr_cookies_banned_total Banned cookies stored\n");
    body.push_str("# TYPE clewdr_cookies_banned_total gauge\n");
    body.push_str(&format!("clewdr_cookies_banned_total {}\n", agg.banned_count));

    body.push_str("# HELP clewdr_requests_total Total requests recorded\n");
    body.push_str("# TYPE clewdr_requests_total counter\n");
    body.push_str(&format!("clewdr_requests_total {}\n", agg.total_requests));

    let headers = [
        (header::CONTENT_TYPE, HeaderValue::from_static("text/plain; version=0.0.4")),
    ];

    Ok((StatusCode::OK, headers, body))
}
