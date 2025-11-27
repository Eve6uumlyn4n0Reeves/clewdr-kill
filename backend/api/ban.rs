use axum::{
    extract::{ConnectInfo, State},
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tracing::{debug, error, info};

use crate::api::response::{success_response, ApiResponse, EmptyResponse};
use crate::{
    claude_web_state::ClaudeWebState,
    config::BanCookie,
    error::ClewdrError,
    middleware::{sanitize_for_log, validate_cookie},
    services::ban_queue::BanQueueInfo,
    VERSION_INFO,
};

#[derive(Deserialize)]
pub struct SubmitCookiePayload {
    pub cookie: String,
}

#[derive(Deserialize)]
pub struct SubmitMultipleCookiesPayload {
    pub cookies: Vec<String>,
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
}

fn default_batch_size() -> usize {
    10
}

#[derive(Serialize)]
pub struct BatchSubmitResult {
    pub success: usize,
    pub failed: usize,
    pub total: usize,
    pub errors: Vec<String>,
}

/// API endpoint to submit a new cookie for banning
pub async fn api_post_cookie(
    State(app_state): State<crate::router::AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(payload): Json<SubmitCookiePayload>,
) -> Result<Json<EmptyResponse>, ClewdrError> {
    let queue = app_state.ban_queue_handle;
    let rate_limiter = app_state.rate_limiter;
    // Check rate limit
    if !rate_limiter.is_allowed(&addr.ip().to_string()).await {
        return Err(ClewdrError::BadRequest {
            msg: "Too many requests, please try again later".into(),
        });
    }

    // Validate and sanitize cookie input
    let validated_cookie = validate_cookie(&payload.cookie).map_err(|e| {
        error!(
            "Cookie validation failed for {}: {}",
            sanitize_for_log(&payload.cookie),
            e
        );
        e
    })?;

    let cookie = BanCookie::new(&validated_cookie).map_err(|e| {
        error!("Invalid cookie format: {}", e);
        ClewdrError::BadRequest {
            msg: "Invalid cookie format".into(),
        }
    })?;

    info!("Cookie accepted for banning: {}", cookie.cookie.ellipse());

    queue.submit(cookie).await.map_err(|e| {
        error!("Failed to submit cookie: {}", e);
        ClewdrError::Whatever {
            message: format!("Failed to submit cookie: {}", e),
            source: Some(Box::new(e)),
        }
    })?;

    Ok(Json(success_response()))
}

/// API endpoint to retrieve all cookies and their status
pub async fn api_get_cookies(
    State(app_state): State<crate::router::AppState>,
) -> Result<Json<ApiResponse<BanQueueInfo>>, ClewdrError> {
    let queue = app_state.ban_queue_handle;

    queue
        .get_status()
        .await
        .map(|status| Json(ApiResponse::success(status)))
        .map_err(|e| {
            error!("Failed to get queue status: {}", e);
            ClewdrError::Whatever {
                message: format!("Failed to get status: {}", e),
                source: Some(Box::new(e)),
            }
        })
}

/// API endpoint to delete a specific cookie
pub async fn api_delete_cookie(
    State(app_state): State<crate::router::AppState>,
    Json(payload): Json<SubmitCookiePayload>,
) -> Result<Json<EmptyResponse>, ClewdrError> {
    let queue = app_state.ban_queue_handle;

    queue.delete(payload.cookie).await.map_err(|e| {
        error!("Failed to delete cookie: {}", e);
        ClewdrError::Whatever {
            message: format!("Failed to delete cookie: {}", e),
            source: Some(Box::new(e)),
        }
    })?;

    info!("Cookie deleted successfully");
    Ok(Json(success_response()))
}

/// API endpoint to submit multiple cookies in parallel
pub async fn api_post_multiple_cookies(
    State(app_state): State<crate::router::AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(payload): Json<SubmitMultipleCookiesPayload>,
) -> Result<Json<ApiResponse<BatchSubmitResult>>, ClewdrError> {
    let queue = app_state.ban_queue_handle;
    let rate_limiter = app_state.rate_limiter;

    // Validate batch size
    if payload.batch_size == 0 || payload.batch_size > 100 {
        return Err(ClewdrError::BadRequest {
            msg: "Batch size must be between 1 and 100".into(),
        });
    }

    // Check rate limit (relaxed for batch operations)
    if !rate_limiter.is_allowed(&addr.ip().to_string()).await {
        return Err(ClewdrError::BadRequest {
            msg: "Too many requests, please try again later".into(),
        });
    }

    info!("Processing batch of {} cookies", payload.cookies.len());

    // Process cookies in chunks to avoid overwhelming the system
    let batch_size = payload.batch_size.min(payload.cookies.len());
    let mut success = 0;
    let mut failed = 0;
    let mut errors = Vec::new();

    for chunk in payload.cookies.chunks(batch_size) {
        let mut tasks = Vec::new();

        // Create parallel tasks for this chunk
        for cookie_str in chunk {
            let queue_clone = queue.clone();
            let cookie_str_clone = cookie_str.clone();

            let task = tokio::spawn(async move {
                // Validate and sanitize cookie
                match validate_cookie(&cookie_str_clone) {
                    Ok(validated_cookie) => match BanCookie::new(&validated_cookie) {
                        Ok(cookie) => match queue_clone.submit(cookie).await {
                            Ok(_) => Ok(()),
                            Err(e) => {
                                error!("Failed to submit cookie: {}", e);
                                Err(format!("Submit failed: {}", e))
                            }
                        },
                        Err(e) => Err(format!("Invalid format: {}", e)),
                    },
                    Err(e) => Err(format!("Validation failed: {}", e)),
                }
            });

            tasks.push(task);
        }

        // Wait for all tasks in this chunk to complete
        for task in tasks {
            match task.await {
                Ok(Ok(())) => success += 1,
                Ok(Err(e)) => {
                    failed += 1;
                    if errors.len() < 10 {
                        // Limit error details to prevent response bloat
                        errors.push(e);
                    }
                }
                Err(e) => {
                    error!("Task failed: {}", e);
                    failed += 1;
                }
            }
        }

        // Small delay between batches to prevent overwhelming
        if chunk.len() == batch_size {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
    }

    info!(
        "Batch submission completed: success={} failed={} total={} first_errors={}",
        success,
        failed,
        payload.cookies.len(),
        errors.iter().take(3).collect::<Vec<_>>().join("; ")
    );

    Ok(Json(ApiResponse::success(BatchSubmitResult {
        success,
        failed,
        total: payload.cookies.len(),
        errors,
    })))
}

/// API endpoint to get the application version information
pub async fn api_version() -> Json<ApiResponse<String>> {
    Json(ApiResponse::success(VERSION_INFO.to_string()))
}

/// Response for cookie check endpoint
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CookieCheckStatus {
    Alive,
    Banned,
    Invalid,
    Error,
}

#[derive(Serialize)]
pub struct CookieCheckResponse {
    pub status: CookieCheckStatus,
    pub alive: bool,
    pub banned: bool,
    #[serde(rename = "lastChecked")]
    pub last_checked: String,
    pub error: Option<String>,
}

/// API endpoint to check if a cookie is alive/banned
pub async fn api_check_cookie(
    Json(payload): Json<SubmitCookiePayload>,
) -> Result<Json<ApiResponse<CookieCheckResponse>>, ClewdrError> {
    // Authentication is handled by middleware

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
            return Ok(Json(ApiResponse::success(CookieCheckResponse {
                status: CookieCheckStatus::Invalid,
                alive: false,
                banned: false,
                last_checked: Utc::now().to_rfc3339(),
                error: Some(format!("Invalid cookie format: {}", e)),
            })));
        }
    };

    // Create minimal state for checking
    let mut state = ClaudeWebState::new_minimal();

    // Try to set the cookie
    if let Err(e) = state.set_cookie(&cookie.cookie.to_string()) {
        return Ok(Json(ApiResponse::success(CookieCheckResponse {
            status: CookieCheckStatus::Error,
            alive: false,
            banned: false,
            last_checked: Utc::now().to_rfc3339(),
            error: Some(format!("Failed to set cookie: {}", e)),
        })));
    }

    // Try to fetch organization UUID to test if cookie is valid
    let check_result = state.fetch_org_uuid().await;

    let response = match check_result {
        Ok(_) => {
            // Successfully got org UUID, cookie is alive
            CookieCheckResponse {
                status: CookieCheckStatus::Alive,
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
                status: if banned {
                    CookieCheckStatus::Banned
                } else {
                    CookieCheckStatus::Error
                },
                alive: false,
                banned,
                last_checked: Utc::now().to_rfc3339(),
                error: Some(error_str),
            }
        }
    };

    Ok(Json(ApiResponse::success(response)))
}
