//! Request logging middleware for API requests
//! Provides structured logging for all HTTP requests and responses

use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};
use std::time::Instant;
use tracing::{info, Instrument};

/// Request logging middleware
pub async fn request_logging_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let start_time = Instant::now();
    // Extract request information
    let method = request.method().to_string();
    let path = request.uri().path().to_string();
    let user_agent = request
        .headers()
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");

    // Check for authorization header
    let has_auth = request.headers().get("authorization").is_some();

    // Create span for request tracing
    let span = tracing::info_span!(
        "http_request",
        method = %method,
        path = %path,
        user_agent = %user_agent,
        has_auth = %has_auth
    );

    // Execute the request within the span
    let response = {
        let span_for_logs = span.clone();
        async move {
            info!(parent: &span_for_logs, "Incoming request");

            let response = next.run(request).await;

            let duration = start_time.elapsed();
            let status = response.status();

            tracing::info!(
                method = %method,
                path = %path,
                status = status.as_u16(),
                duration_ms = duration.as_millis(),
                "API response"
            );

            info!(
                parent: &span_for_logs,
                status = %status,
                duration_ms = duration.as_millis(),
                "Request completed"
            );

            response
        }
    }
    .instrument(span)
    .await;

    Ok(response)
}

/// Log security-related events
pub fn log_security_event(event_type: &str, client_ip: Option<&str>, details: &str) {
    tracing::warn!(
        event_type = event_type,
        client_ip = client_ip.unwrap_or("unknown"),
        details = details,
        "Security event"
    );
}

/// Log authentication attempts
pub fn log_auth_attempt(client_ip: Option<&str>, success: bool, reason: Option<&str>) {
    if success {
        tracing::info!(
            client_ip = client_ip.unwrap_or("unknown"),
            "Authentication successful"
        );
    } else {
        tracing::warn!(
            client_ip = client_ip.unwrap_or("unknown"),
            reason = reason.unwrap_or("unknown"),
            "Authentication failed"
        );
    }
}
