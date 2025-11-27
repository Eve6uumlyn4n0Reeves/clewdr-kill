use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use crate::api::response::ApiResponse;
use crate::error::ClewdrError;
use crate::router::AppState;
use crate::utils::logging::audit_log;
use std::{sync::LazyLock, time::Instant};

static START_TIME: LazyLock<Instant> = LazyLock::new(Instant::now);

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminAction {
    pub action: AdminActionType,
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminActionType {
    PauseAll,
    ResumeAll,
    ResetStats,
    ClearAll,
    EmergencyStop,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminActionResult {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Clone)]
pub struct SystemStatus {
    pub status: SystemState,
    pub uptime_seconds: u64,
    pub active_workers: u32,
    pub queue_size: u32,
    pub banned_count: u32,
    pub total_requests: u64,
    pub error_count: u64,
    pub last_error: Option<String>,
    pub maintenance_mode: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum SystemState {
    Starting,
    Running,
    Paused,
    Stopping,
    Error,
    Maintenance,
}

impl SystemStatus {
    pub fn new() -> Self {
        Self {
            status: SystemState::Starting,
            uptime_seconds: 0,
            active_workers: 0,
            queue_size: 0,
            banned_count: 0,
            total_requests: 0,
            error_count: 0,
            last_error: None,
            maintenance_mode: false,
        }
    }
}

// Global system status
static SYSTEM_STATUS: std::sync::OnceLock<std::sync::RwLock<SystemStatus>> =
    std::sync::OnceLock::new();

/// Execute administrative action
pub async fn api_execute_admin_action(
    State(app_state): State<AppState>,
    Json(action): Json<AdminAction>,
) -> Result<Json<ApiResponse<AdminActionResult>>, ClewdrError> {
    let result = match action.action {
        AdminActionType::PauseAll => pause_all_workers(&app_state).await.map_err(|e| {
            tracing::error!("Failed to pause all workers: {}", e);
            e
        })?,
        AdminActionType::ResumeAll => resume_all_workers(&app_state).await.map_err(|e| {
            tracing::error!("Failed to resume all workers: {}", e);
            e
        })?,
        AdminActionType::ResetStats => reset_statistics(&app_state).await,
        AdminActionType::ClearAll => clear_all_queues(&app_state).await,
        AdminActionType::EmergencyStop => emergency_stop(&app_state).await.map_err(|e| {
            tracing::error!("Failed to execute emergency stop: {}", e);
            e
        })?,
    };

    Ok(Json(ApiResponse::success(result)))
}

/// Get current system status
pub async fn api_get_system_status(
    State(app_state): State<AppState>,
) -> Result<Json<ApiResponse<SystemStatus>>, ClewdrError> {
    let queue_info = app_state.ban_queue_handle.get_status().await.map_err(|e| {
        tracing::error!("Failed to get queue status: {}", e);
        ClewdrError::Whatever {
            message: "Failed to get queue status".to_string(),
            source: Some(Box::new(e)),
        }
    })?;

    let status = SystemStatus {
        status: SystemState::Running,
        uptime_seconds: START_TIME.elapsed().as_secs(),
        active_workers: app_state.ban_farm.worker_count().await as u32,
        queue_size: queue_info.pending.len() as u32,
        banned_count: queue_info.banned.len() as u32,
        total_requests: queue_info.total_requests,
        error_count: 0,
        last_error: None,
        maintenance_mode: false,
    };

    audit_log(
        "admin_status",
        None,
        &format!(
            "workers={} queue_pending={} banned={}",
            status.active_workers, status.queue_size, status.banned_count
        ),
    );

    update_system_status(|s| {
        *s = status.clone();
    })
    .await
    .map_err(|e| {
        tracing::error!("Failed to update system status: {}", e);
        ClewdrError::InternalServerError {
            msg: format!("System status update failed: {}", e),
        }
    })?;

    Ok(Json(ApiResponse::success(status)))
}

/// Get detailed health check
pub async fn api_health_check(
    State(app_state): State<AppState>,
) -> Result<Json<ApiResponse<HealthCheckResult>>, ClewdrError> {
    // Authentication is handled by middleware
    let health_result = perform_health_check(&app_state).await;
    Ok(Json(ApiResponse::success(health_result)))
}

#[derive(Debug, Serialize)]
pub struct HealthCheckResult {
    pub overall_status: HealthStatus,
    pub components: Vec<ComponentHealth>,
    pub recommendations: Vec<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
    Unknown,
}

#[derive(Debug, Serialize)]
pub struct ComponentHealth {
    pub name: String,
    pub status: HealthStatus,
    pub message: Option<String>,
    pub metrics: Option<serde_json::Value>,
    pub last_check: chrono::DateTime<chrono::Utc>,
}

// Action implementations
async fn pause_all_workers(app_state: &AppState) -> Result<AdminActionResult, ClewdrError> {
    app_state.ban_farm.pause().await;
    update_system_status(|status| {
        status.status = SystemState::Paused;
    })
    .await?;

    Ok(AdminActionResult {
        success: true,
        message: "已暂停所有 worker".to_string(),
        data: Some(serde_json::json!({
            "paused_at": chrono::Utc::now().to_rfc3339()
        })),
        timestamp: chrono::Utc::now(),
    })
}

async fn resume_all_workers(app_state: &AppState) -> Result<AdminActionResult, ClewdrError> {
    app_state.ban_farm.resume().await;
    update_system_status(|status| {
        status.status = SystemState::Running;
    })
    .await?;

    Ok(AdminActionResult {
        success: true,
        message: "已恢复所有 worker".to_string(),
        data: Some(serde_json::json!({
            "resumed_at": chrono::Utc::now().to_rfc3339()
        })),
        timestamp: chrono::Utc::now(),
    })
}

async fn reset_statistics(app_state: &AppState) -> AdminActionResult {
    if let Err(e) = app_state.ban_queue_handle.reset_stats().await {
        return AdminActionResult {
            success: false,
            message: format!("Failed to reset queue stats: {e}"),
            data: None,
            timestamp: chrono::Utc::now(),
        };
    }
    app_state.ban_farm.reset_strategy_metrics().await;

    AdminActionResult {
        success: true,
        message: "Statistics have been reset".to_string(),
        data: Some(serde_json::json!({
            "reset_at": chrono::Utc::now().to_rfc3339()
        })),
        timestamp: chrono::Utc::now(),
    }
}

async fn clear_all_queues(app_state: &AppState) -> AdminActionResult {
    if let Err(e) = app_state.ban_queue_handle.clear_all().await {
        return AdminActionResult {
            success: false,
            message: format!("Failed to clear all queues: {e}"),
            data: None,
            timestamp: chrono::Utc::now(),
        };
    }
    AdminActionResult {
        success: true,
        message: "All queues have been cleared".to_string(),
        data: Some(serde_json::json!({
            "cleared_at": chrono::Utc::now().to_rfc3339()
        })),
        timestamp: chrono::Utc::now(),
    }
}

async fn emergency_stop(app_state: &AppState) -> Result<AdminActionResult, ClewdrError> {
    app_state.ban_farm.stop().await;
    update_system_status(|status| {
        status.status = SystemState::Stopping;
        status.maintenance_mode = true;
    })
    .await?;

    Ok(AdminActionResult {
        success: true,
        message: "紧急停止：worker 将不再执行任务".to_string(),
        data: Some(serde_json::json!({
            "stopped_at": chrono::Utc::now().to_rfc3339()
        })),
        timestamp: chrono::Utc::now(),
    })
}

async fn update_system_status<F>(updater: F) -> Result<(), ClewdrError>
where
    F: FnOnce(&mut SystemStatus),
{
    let status = SYSTEM_STATUS.get_or_init(|| std::sync::RwLock::new(SystemStatus::new()));
    let mut status_guard = status.write().map_err(|e| {
        tracing::error!("Failed to acquire write lock for system status: {}", e);
        ClewdrError::InternalServerError {
            msg: format!("System status lock poisoned: {}", e),
        }
    })?;
    updater(&mut status_guard);
    status_guard.uptime_seconds = get_uptime_seconds();
    Ok(())
}

async fn perform_health_check(app_state: &AppState) -> HealthCheckResult {
    let mut components = Vec::new();
    let queue_info = app_state.ban_queue_handle.get_status().await.ok();

    // Check queue health
    components.push(ComponentHealth {
        name: "Ban Queue".to_string(),
        status: queue_info
            .as_ref()
            .map(|q| {
                if q.pending.is_empty() {
                    HealthStatus::Healthy
                } else {
                    HealthStatus::Healthy
                }
            })
            .unwrap_or(HealthStatus::Warning),
        message: queue_info
            .as_ref()
            .map(|q| format!("Pending: {}, Banned: {}", q.pending.len(), q.banned.len()))
            .or_else(|| Some("Queue status unavailable".to_string())),
        metrics: queue_info.as_ref().map(|q| {
            serde_json::json!({
                "queue_size": q.pending.len(),
                "banned_size": q.banned.len(),
                "total_requests": q.total_requests,
            })
        }),
        last_check: chrono::Utc::now(),
    });

    // Check worker health
    components.push(ComponentHealth {
        name: "Workers".to_string(),
        status: HealthStatus::Healthy,
        message: Some("Workers running".to_string()),
        metrics: Some(serde_json::json!({
            "active_workers": app_state.ban_farm.worker_count().await
        })),
        last_check: chrono::Utc::now(),
    });

    // Check security manager
    // Skip unused security manager / memory placeholders

    let overall_status = if components
        .iter()
        .any(|c| matches!(c.status, HealthStatus::Critical))
    {
        HealthStatus::Critical
    } else if components
        .iter()
        .any(|c| matches!(c.status, HealthStatus::Warning))
    {
        HealthStatus::Warning
    } else if components
        .iter()
        .all(|c| matches!(c.status, HealthStatus::Healthy))
    {
        HealthStatus::Healthy
    } else {
        HealthStatus::Unknown
    };

    let recommendations = if matches!(overall_status, HealthStatus::Warning) {
        vec![
            "Consider increasing memory allocation".to_string(),
            "Monitor queue processing speed".to_string(),
        ]
    } else {
        vec![]
    };

    HealthCheckResult {
        overall_status,
        components,
        recommendations,
        timestamp: chrono::Utc::now(),
    }
}

fn get_uptime_seconds() -> u64 {
    START_TIME.elapsed().as_secs()
}
