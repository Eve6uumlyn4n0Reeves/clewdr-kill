//! Input validation module for security
//! Provides validation functions to prevent injection attacks

use crate::error::ClewdrError;
use regex::Regex;
use std::sync::LazyLock;

/// Maximum length for cookie strings
const MAX_COOKIE_LENGTH: usize = 200;
/// Maximum length for configuration values
const MAX_CONFIG_VALUE_LENGTH: usize = 1000;
/// Maximum length for prompt names
const MAX_PROMPT_NAME_LENGTH: usize = 100;
/// Maximum length for file paths
const MAX_PATH_LENGTH: usize = 4096;

static COOKIE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[\x20-\x7E]*$").unwrap() // ASCII printable characters only
});

static PATH_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9_\-./\\]+$").unwrap());

static CONFIG_KEY_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z][a-zA-Z0-9_]*$").unwrap());

/// Validates cookie string for security
pub fn validate_cookie(cookie: &str) -> Result<String, ClewdrError> {
    // Length check
    if cookie.len() > MAX_COOKIE_LENGTH {
        return Err(ClewdrError::BadRequest {
            msg: format!("Cookie too long (max {} chars)", MAX_COOKIE_LENGTH),
        });
    }

    if cookie.is_empty() {
        return Err(ClewdrError::BadRequest {
            msg: "Cookie cannot be empty".into(),
        });
    }

    // ASCII printable characters only
    if !COOKIE_REGEX.is_match(cookie) {
        return Err(ClewdrError::BadRequest {
            msg: "Cookie contains invalid characters".into(),
        });
    }

    // Remove potential SQL injection patterns
    let suspicious_patterns = [
        "SELECT", "INSERT", "UPDATE", "DELETE", "DROP", "UNION", "--", "/*", "*/", ";", "'", "\"",
        "xp_", "sp_", "OR", "AND", "WHERE", "EXEC", "EXECUTE",
    ];

    let upper_cookie = cookie.to_uppercase();
    for pattern in &suspicious_patterns {
        if upper_cookie.contains(pattern) {
            tracing::warn!("Suspicious pattern in cookie: {}", pattern);
            return Err(ClewdrError::BadRequest {
                msg: "Invalid cookie format".into(),
            });
        }
    }

    Ok(cookie.to_string())
}

/// Validates configuration key
pub fn validate_config_key(key: &str) -> Result<String, ClewdrError> {
    if key.len() > MAX_PROMPT_NAME_LENGTH {
        return Err(ClewdrError::BadRequest {
            msg: format!("Key too long (max {} chars)", MAX_PROMPT_NAME_LENGTH),
        });
    }

    if !CONFIG_KEY_REGEX.is_match(key) {
        return Err(ClewdrError::BadRequest {
            msg: "Key contains invalid characters".into(),
        });
    }

    Ok(key.to_string())
}

/// Validates configuration value
pub fn validate_config_value(value: &str) -> Result<String, ClewdrError> {
    if value.len() > MAX_CONFIG_VALUE_LENGTH {
        return Err(ClewdrError::BadRequest {
            msg: format!("Value too long (max {} chars)", MAX_CONFIG_VALUE_LENGTH),
        });
    }

    // Check for potential code injection
    let dangerous_patterns = ["${", "<script", "javascript:", "data:", "vbscript:"];
    let lower_value = value.to_lowercase();

    for pattern in &dangerous_patterns {
        if lower_value.contains(pattern) {
            return Err(ClewdrError::BadRequest {
                msg: "Value contains potentially dangerous content".into(),
            });
        }
    }

    Ok(value.to_string())
}

/// Validates file path
pub fn validate_path(path: &str) -> Result<String, ClewdrError> {
    if path.len() > MAX_PATH_LENGTH {
        return Err(ClewdrError::BadRequest {
            msg: format!("Path too long (max {} chars)", MAX_PATH_LENGTH),
        });
    }

    if path.is_empty() {
        return Err(ClewdrError::BadRequest {
            msg: "Path cannot be empty".into(),
        });
    }

    // Prevent path traversal
    if path.contains("..") || path.contains("~") {
        return Err(ClewdrError::BadRequest {
            msg: "Path traversal not allowed".into(),
        });
    }

    if !PATH_REGEX.is_match(path) {
        return Err(ClewdrError::BadRequest {
            msg: "Path contains invalid characters".into(),
        });
    }

    Ok(path.to_string())
}

/// Validates prompt name
pub fn validate_prompt_name(name: &str) -> Result<String, ClewdrError> {
    if name.len() > MAX_PROMPT_NAME_LENGTH {
        return Err(ClewdrError::BadRequest {
            msg: format!("Name too long (max {} chars)", MAX_PROMPT_NAME_LENGTH),
        });
    }

    if name.is_empty() {
        return Err(ClewdrError::BadRequest {
            msg: "Name cannot be empty".into(),
        });
    }

    // No control characters
    if name.chars().any(|c| c.is_control()) {
        return Err(ClewdrError::BadRequest {
            msg: "Name contains invalid characters".into(),
        });
    }

    // Prevent file system injection
    let forbidden = ["/", "\\", ":", "*", "?", "\"", "<", ">", "|"];
    for char in &forbidden {
        if name.contains(char) {
            return Err(ClewdrError::BadRequest {
                msg: "Name contains forbidden characters".into(),
            });
        }
    }

    Ok(name.to_string())
}

/// Validates prompt content
pub fn validate_prompt_content(content: &str) -> Result<String, ClewdrError> {
    // Reasonable size limit for prompts
    const MAX_PROMPT_SIZE: usize = 100_000; // 100KB

    if content.len() > MAX_PROMPT_SIZE {
        return Err(ClewdrError::BadRequest {
            msg: format!("Prompt too large (max {} chars)", MAX_PROMPT_SIZE),
        });
    }

    Ok(content.to_string())
}

/// Sanitizes string for logging (removes sensitive data)
pub fn sanitize_for_log(input: &str) -> String {
    if input.len() <= 10 {
        return "[REDACTED]".to_string();
    }
    format!("{}...[REDACTED]", &input[..10])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_cookie() {
        assert!(validate_cookie("valid-cookie-123").is_ok());
        assert!(validate_cookie("cookie with space").is_ok());
        assert!(validate_cookie("").is_err());
        assert!(validate_cookie("cookie'; DROP TABLE users; --").is_err());
    }

    #[test]
    fn test_validate_path() {
        assert!(validate_path("/valid/path").is_ok());
        assert!(validate_path("../etc/passwd").is_err());
        assert!(validate_path("").is_err());
    }
}
