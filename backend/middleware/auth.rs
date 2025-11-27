use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use axum_auth::AuthBearer;
use tracing::debug;

use crate::{error::ClewdrError, router::AppState};

/// Admin authentication middleware
/// This middleware checks if the request has a valid admin token
/// If authentication fails, it returns an error immediately
/// If authentication succeeds, it passes the request to the next handler
pub async fn admin_auth_middleware(
    State(app_state): State<AppState>,
    AuthBearer(token): AuthBearer,
    mut request: Request,
    next: Next,
) -> Result<Response, ClewdrError> {
    debug!("Checking admin authentication");

    let claims = app_state.token_manager.validate(&token)?;
    request.extensions_mut().insert(claims);

    debug!("Admin authentication successful");
    Ok(next.run(request).await)
}

/// Authentication error response helper
pub fn auth_error_response() -> ClewdrError {
    ClewdrError::InvalidAuth
}
