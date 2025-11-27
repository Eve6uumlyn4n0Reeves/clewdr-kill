use axum::{extract::State, Json};
use serde::Deserialize;
use tracing::error;

use crate::{
    api::response::{success_response, EmptyResponse},
    error::ClewdrError,
    router::AppState,
};

#[derive(Debug, Deserialize)]
pub struct FrontendErrorReport {
    pub message: String,
    pub stack: Option<String>,
    pub component_stack: Option<String>,
    pub url: String,
    pub user_agent: String,
    pub timestamp: String,
}

#[derive(Debug, Deserialize)]
pub struct ErrorReportPayload {
    pub errors: Vec<FrontendErrorReport>,
}

/// 接收前端错误报告并记录到审计日志
pub async fn api_report_frontend_errors(
    State(_app_state): State<AppState>,
    Json(payload): Json<ErrorReportPayload>,
) -> Result<Json<EmptyResponse>, ClewdrError> {
    for err in payload.errors {
        // 记录到审计日志,方便后续排查
        error!(
            audit = true,
            alert = "frontend_error",
            message = %err.message,
            url = %err.url,
            user_agent = %err.user_agent,
            timestamp = %err.timestamp,
            stack = ?err.stack,
            component_stack = ?err.component_stack,
            "Frontend error reported: {}",
            err.message
        );
    }

    Ok(Json(success_response()))
}
