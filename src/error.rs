use std::fmt::Display;

use axum::{Json, extract::rejection::JsonRejection, response::IntoResponse};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use snafu::Location;
use strum::IntoStaticStr;
use tracing::debug;
use wreq::{Response, StatusCode, header::InvalidHeaderValue};

#[derive(Debug, IntoStaticStr, snafu::Snafu)]
#[snafu(visibility(pub(crate)))]
#[strum(serialize_all = "snake_case")]
pub enum ClewdrError {
    #[snafu(display("Parse cookie error: {}, at: {}", msg, loc))]
    ParseCookieError {
        #[snafu(implicit)]
        loc: Location,
        msg: &'static str,
    },
    #[snafu(display("Ractor error: {}", msg))]
    RactorError {
        #[snafu(implicit)]
        loc: Location,
        msg: String,
    },
    #[snafu(display("URL parse error: {}, at: {}", source, loc))]
    UrlError {
        #[snafu(implicit)]
        loc: Location,
        source: url::ParseError,
    },
    #[snafu(display("Invalid header value: {}", source))]
    #[snafu(context(false))]
    InvalidHeaderValue { source: InvalidHeaderValue },
    #[snafu(display("Bad request: {}", msg))]
    BadRequest { msg: &'static str },
    #[snafu(display("No cookie available"))]
    NoCookieAvailable,
    #[snafu(display("Failed to parse TOML: {}", source))]
    #[snafu(context(false))]
    TomlDeError { source: toml::de::Error },
    #[snafu(transparent)]
    TomlSeError { source: toml::ser::Error },
    #[snafu(transparent)]
    JsonRejection { source: JsonRejection },
    #[snafu(display("Request error: {}, source: {}", msg, source))]
    WreqError {
        msg: &'static str,
        source: wreq::Error,
    },
    #[snafu(display("HTTP error: code: {}, body: {}", code.to_string().red(), inner.to_string()))]
    ClaudeHttpError {
        code: StatusCode,
        inner: ClaudeErrorBody,
    },
    #[snafu(display("Unexpected None: {}", msg))]
    UnexpectedNone { msg: &'static str },
    #[snafu(display("IO error: {}", source))]
    #[snafu(context(false))]
    IoError {
        #[snafu(implicit)]
        loc: Location,
        source: std::io::Error,
    },
    #[snafu(display("{}", msg))]
    PathNotFound { msg: String },
    #[snafu(display("Key/Password Invalid"))]
    InvalidAuth,
    #[snafu(whatever, display("{}: {}", message, source.as_ref().map_or_else(|| "Unknown error".into(), |e| e.to_string())))]
    Whatever {
        message: String,
        #[snafu(source(from(Box<dyn std::error::Error + Send>, Some)))]
        source: Option<Box<dyn std::error::Error + Send>>,
    },
}

impl IntoResponse for ClewdrError {
    fn into_response(self) -> axum::response::Response {
        let (status, msg) = match self {
            ClewdrError::UrlError { source, .. } => (
                StatusCode::BAD_REQUEST,
                json!(format!("URL parse error: {}", source)),
            ),
            ClewdrError::ParseCookieError { .. } => {
                (StatusCode::BAD_REQUEST, json!(self.to_string()))
            }
            ClewdrError::ClaudeHttpError { code, inner } => {
                return (code, Json(ClaudeError { error: inner })).into_response();
            }
            ClewdrError::JsonRejection { ref source } => {
                (source.status(), json!(source.body_text()))
            }
            ClewdrError::PathNotFound { .. } => (StatusCode::NOT_FOUND, json!(self.to_string())),
            ClewdrError::InvalidAuth => (StatusCode::UNAUTHORIZED, json!(self.to_string())),
            ClewdrError::BadRequest { .. } => (StatusCode::BAD_REQUEST, json!(self.to_string())),
            ClewdrError::InvalidHeaderValue { .. } => {
                (StatusCode::BAD_REQUEST, json!(self.to_string()))
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, json!(self.to_string())),
        };
        let err = ClaudeError {
            error: ClaudeErrorBody {
                message: msg,
                r#type: <&str>::from(self).into(),
                code: Some(status.as_u16()),
            },
        };
        (status, Json(err)).into_response()
    }
}

/// HTTP error response
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClaudeError {
    pub error: ClaudeErrorBody,
}

/// Inner HTTP error response
#[derive(Debug, Serialize, Clone)]
pub struct ClaudeErrorBody {
    pub message: Value,
    pub r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<u16>,
}

/// Raw Inner HTTP error response
#[derive(Debug, Deserialize)]
struct RawBody {
    pub message: String,
    pub r#type: String,
}

impl<'de> Deserialize<'de> for ClaudeErrorBody {
    /// when message is a json string, try parse it as a object
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = RawBody::deserialize(deserializer)?;
        if let Ok(message) = serde_json::from_str::<Value>(&raw.message) {
            return Ok(ClaudeErrorBody {
                message,
                r#type: raw.r#type,
                code: None,
            });
        }
        Ok(ClaudeErrorBody {
            message: json!(raw.message),
            r#type: raw.r#type,
            code: None,
        })
    }
}

impl Display for ClaudeErrorBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        serde_json::to_string_pretty(self)
            .map_err(|_| std::fmt::Error)?
            .fmt(f)
    }
}

pub trait CheckClaudeErr
where
    Self: Sized,
{
    fn check_claude(self) -> impl Future<Output = Result<Self, ClewdrError>>;
}

impl CheckClaudeErr for Response {
    /// Checks response from Claude Web API for ban detection
    /// Simplified error checking focused on detecting account bans
    async fn check_claude(self) -> Result<Self, ClewdrError> {
        let status = self.status();
        if status.is_success() {
            return Ok(self);
        }

        debug!("Error response status: {}", status);

        // Check for common ban indicators
        if status == 302 {
            // blocked by cloudflare - likely banned
            let error = ClaudeErrorBody {
                message: json!("Blocked, likely banned"),
                r#type: "banned".to_string(),
                code: Some(status.as_u16()),
            };
            return Err(ClewdrError::ClaudeHttpError {
                code: status,
                inner: error,
            });
        }

        let text = match self.text().await {
            Ok(text) => text,
            Err(err) => {
                let error = ClaudeErrorBody {
                    message: json!(err.to_string()),
                    r#type: "error_get_error_body".to_string(),
                    code: Some(status.as_u16()),
                };
                return Err(ClewdrError::ClaudeHttpError {
                    code: status,
                    inner: error,
                });
            }
        };

        // Try to parse as Claude error
        if let Ok(err) = serde_json::from_str::<ClaudeError>(&text) {
            // Check for account disabled message
            if status == 400 && err.error.message == json!("This organization has been disabled.") {
                return Err(ClewdrError::BadRequest {
                    msg: "Account disabled",
                });
            }

            // Check for authentication errors (likely banned)
            if status == 401 || status == 403 {
                return Err(ClewdrError::BadRequest {
                    msg: "Authentication failed - likely banned",
                });
            }

            let inner_error = err.error;
            return Err(ClewdrError::ClaudeHttpError {
                code: status,
                inner: inner_error,
            });
        }

        // Default error for unparsable responses
        let error = ClaudeErrorBody {
            message: format!("HTTP {}: {}", status, text).into(),
            r#type: "http_error".to_string(),
            code: Some(status.as_u16()),
        };
        Err(ClewdrError::ClaudeHttpError {
            code: status,
            inner: error,
        })
    }
}
