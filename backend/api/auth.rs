use axum::{
    extract::{ConnectInfo, State},
    Json,
};
use axum_auth::AuthBearer;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

use super::response::{success_response, ApiResponse, EmptyResponse};
use crate::{
    config::CLEWDR_CONFIG,
    error::ClewdrError,
    router::AppState,
    utils::logging::audit_log,
};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub expires_at: String,
}

pub async fn api_login(
    State(app_state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<ApiResponse<LoginResponse>>, ClewdrError> {
    if !app_state
        .login_rate_limiter
        .is_allowed(&addr.ip().to_string())
        .await
    {
        audit_log("login_blocked_rate_limit", Some(&addr.ip().to_string()), "too_many_login");
        return Err(ClewdrError::BadRequest {
            msg: "Too many login attempts, please try again later".into(),
        });
    }

    if !CLEWDR_CONFIG.load().admin_auth(payload.password.trim()) {
        audit_log("login_failed", Some(&addr.ip().to_string()), "invalid_password");
        return Err(ClewdrError::InvalidAuth);
    }

    let issued = app_state.token_manager.issue("admin")?;
    audit_log("login_success", Some(&addr.ip().to_string()), "admin");

    Ok(Json(ApiResponse::success(LoginResponse {
        token: issued.token,
        expires_at: issued.expires_at.to_rfc3339(),
    })))
}

pub async fn api_validate_token(
    State(app_state): State<AppState>,
    AuthBearer(token): AuthBearer,
) -> Result<Json<EmptyResponse>, ClewdrError> {
    app_state.token_manager.validate(&token)?;
    Ok(Json(success_response()))
}
