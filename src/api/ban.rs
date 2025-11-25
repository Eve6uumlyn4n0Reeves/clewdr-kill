use axum::{Json, extract::{ConnectInfo, State}};
use axum_auth::AuthBearer;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tracing::{debug, error, info};
use wreq::StatusCode;

use super::error::ApiError;
use crate::{
    VERSION_INFO,
    claude_web_state::ClaudeWebState,
    config::{BanCookie, CLEWDR_CONFIG},
    services::ban_queue::BanQueueInfo,
};

#[derive(Deserialize)]
pub struct SubmitCookiePayload {
    pub cookie: String,
}

/// API endpoint to submit a new cookie for banning
pub async fn api_post_cookie(
    State(app_state): State<crate::router::AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    AuthBearer(token): AuthBearer,
    Json(payload): Json<SubmitCookiePayload>,
) -> Result<StatusCode, ApiError> {
    let queue = app_state.ban_queue_handle;
    let rate_limiter = app_state.rate_limiter;
    // Check rate limit
    if !rate_limiter.is_allowed(&addr.ip().to_string()).await {
        return Err(ApiError::too_many_requests());
    }

    if !CLEWDR_CONFIG.load().admin_auth(&token) {
        return Err(ApiError::unauthorized());
    }

    let cookie = BanCookie::new(&payload.cookie).map_err(|e| {
        error!("Invalid cookie format: {}", e);
        ApiError::bad_request("Invalid cookie format")
    })?;

    info!("Cookie accepted for banning: {}", cookie.cookie.ellipse());

    queue.submit(cookie).await.map_err(|e| {
        error!("Failed to submit cookie: {}", e);
        ApiError::internal(format!("Failed to submit cookie: {}", e))
    })?;

    Ok(StatusCode::OK)
}

/// API endpoint to retrieve all cookies and their status
pub async fn api_get_cookies(
    State(app_state): State<crate::router::AppState>,
    AuthBearer(token): AuthBearer,
) -> Result<Json<BanQueueInfo>, ApiError> {
    let queue = app_state.ban_queue_handle;
    if !CLEWDR_CONFIG.load().admin_auth(&token) {
        return Err(ApiError::unauthorized());
    }

    queue.get_status().await.map(Json).map_err(|e| {
        error!("Failed to get queue status: {}", e);
        ApiError::internal(format!("Failed to get status: {}", e))
    })
}

/// API endpoint to delete a specific cookie
pub async fn api_delete_cookie(
    State(app_state): State<crate::router::AppState>,
    AuthBearer(token): AuthBearer,
    Json(payload): Json<SubmitCookiePayload>,
) -> Result<StatusCode, ApiError> {
    let queue = app_state.ban_queue_handle;
    if !CLEWDR_CONFIG.load().admin_auth(&token) {
        return Err(ApiError::unauthorized());
    }

    queue.delete(payload.cookie).await.map_err(|e| {
        error!("Failed to delete cookie: {}", e);
        ApiError::internal(format!("Failed to delete cookie: {}", e))
    })?;

    info!("Cookie deleted successfully");
    Ok(StatusCode::NO_CONTENT)
}

/// API endpoint to get the application version information
pub async fn api_version() -> String {
    VERSION_INFO.to_string()
}

/// API endpoint to verify authentication
pub async fn api_auth(AuthBearer(token): AuthBearer) -> StatusCode {
    if !CLEWDR_CONFIG.load().admin_auth(&token) {
        return StatusCode::UNAUTHORIZED;
    }
    info!("Auth token accepted");
    StatusCode::OK
}

/// Response for cookie check endpoint
#[derive(Serialize)]
pub struct CookieCheckResponse {
    pub alive: bool,
    pub banned: bool,
    pub last_checked: String,
    pub error: Option<String>,
}

/// API endpoint to check if a cookie is alive/banned
pub async fn api_check_cookie(
    AuthBearer(token): AuthBearer,
    Json(payload): Json<SubmitCookiePayload>,
) -> Result<Json<CookieCheckResponse>, ApiError> {
    if !CLEWDR_CONFIG.load().admin_auth(&token) {
        return Err(ApiError::unauthorized());
    }

    debug!(
        "Checking cookie status: {}",
        BanCookie::new(&payload.cookie)
            .map(|c| c.cookie.ellipse())
            .unwrap_or_else(|_| "invalid".to_string())
    );

    // Parse and validate cookie
    let cookie = match BanCookie::new(&payload.cookie) {
        Ok(cookie) => cookie,
        Err(e) => {
            return Ok(Json(CookieCheckResponse {
                alive: false,
                banned: false,
                last_checked: Utc::now().to_rfc3339(),
                error: Some(format!("Invalid cookie format: {}", e)),
            }));
        }
    };

    // Create minimal state for checking
    let mut state = ClaudeWebState::new_minimal();

    // Try to set the cookie
    if let Err(e) = state.set_cookie(&cookie.cookie.to_string()) {
        return Ok(Json(CookieCheckResponse {
            alive: false,
            banned: false,
            last_checked: Utc::now().to_rfc3339(),
            error: Some(format!("Failed to set cookie: {}", e)),
        }));
    }

    // Try to fetch organization UUID to test if cookie is valid
    let check_result = state.fetch_org_uuid().await;

    let response = match check_result {
        Ok(_) => {
            // Successfully got org UUID, cookie is alive
            CookieCheckResponse {
                alive: true,
                banned: false,
                last_checked: Utc::now().to_rfc3339(),
                error: None,
            }
        }
        Err(e) => {
            let error_str = e.to_string();
            let banned = error_str.contains("banned")
                || error_str.contains("disabled")
                || error_str.contains("403")
                || error_str.contains("401");

            CookieCheckResponse {
                alive: false,
                banned,
                last_checked: Utc::now().to_rfc3339(),
                error: Some(error_str),
            }
        }
    };

    Ok(Json(response))
}
