use serde::Serialize;
use strum::IntoStaticStr;

/// 统一错误码枚举（前后端共用）
#[derive(Debug, Clone, Copy, IntoStaticStr, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    AuthFailed,
    AuthRateLimited,
    InvalidInput,
    CookieFormatInvalid,
    CookieDuplicate,
    RateLimited,
    PromptMissing,
    PromptIoError,
    ClaudeError,
    ClaudeRateLimited,
    ClaudeBanned,
    DbError,
    ConfigInvalid,
    ConfigSaveFailed,
    NotFound,
    Internal,
}

/// Standard API response wrapper
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ApiError>,
}

/// Standard error information
#[derive(Debug, Serialize)]
pub struct ApiError {
    pub message: String,
    pub code: ErrorCode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl<T> ApiResponse<T> {
    /// Create a successful response
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    /// Create an error response
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(ApiError {
                message: message.into(),
                code: ErrorCode::Internal,
                details: None,
            }),
        }
    }

    /// Create an error response with status code
    pub fn error_with_code(message: impl Into<String>, code: ErrorCode) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(ApiError {
                message: message.into(),
                code,
                details: None,
            }),
        }
    }
}

/// For endpoints that don't return data, just success status
pub type EmptyResponse = ApiResponse<serde_json::Value>;

/// Helper to create a simple success response
pub fn success_response() -> EmptyResponse {
    ApiResponse::success(serde_json::Value::Null)
}

/// Helper to create a simple success response with message
pub fn success_message(message: impl Into<String>) -> EmptyResponse {
    ApiResponse::success(serde_json::json!({ "message": message.into() }))
}
