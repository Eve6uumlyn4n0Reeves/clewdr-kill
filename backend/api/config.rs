use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use crate::api::response::ApiResponse;
use crate::error::ClewdrError;
use crate::{
    config::{BanConfig, CLEWDR_CONFIG},
    router::AppState,
};
use crate::utils::logging::audit_log;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigResponse {
    pub ban_config: BanConfig,
    pub server_config: ServerConfig,
    pub network_config: NetworkConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub ip: String,
    pub port: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub proxy: Option<String>,
    pub admin_password: String, // Note: In production, never return the password
}

#[derive(Debug, Deserialize)]
pub struct ConfigUpdateRequest {
    pub ban_config: Option<BanConfig>,
    pub server_config: Option<ServerConfig>,
    pub network_config: Option<NetworkConfig>,
}

/// Shared validation logic for ban and server configuration.
fn validate_ban_and_server(
    ban_config: Option<&BanConfig>,
    server_config: Option<&ServerConfig>,
) -> ValidationResult {
    let mut validation_result = ValidationResult {
        valid: true,
        errors: Vec::new(),
        warnings: Vec::new(),
    };

    // Validate ban configuration
    if let Some(ban_config) = ban_config {
        if ban_config.concurrency == 0 {
            validation_result.valid = false;
            validation_result.errors.push("并发数必须大于0".to_string());
        } else if ban_config.concurrency > 50 {
            validation_result
                .warnings
                .push("并发数过高可能导致API限制".to_string());
        }

        if ban_config.pause_seconds < 60 {
            validation_result
                .warnings
                .push("暂停时间过短可能导致频繁限制".to_string());
        }

        if ban_config.max_tokens < 100 {
            validation_result
                .warnings
                .push("Token数量过少可能影响效果".to_string());
        }

        if ban_config.models.is_empty() {
            validation_result.valid = false;
            validation_result
                .errors
                .push("必须至少配置一个模型".to_string());
        }
    }

    // Validate server configuration
    if let Some(server_config) = server_config {
        if server_config.port == 0 {
            validation_result.valid = false;
            validation_result.errors.push("无效的端口号".to_string());
        }

        if server_config.ip.parse::<std::net::IpAddr>().is_err() {
            validation_result.valid = false;
            validation_result.errors.push("无效的IP地址".to_string());
        }
    }

    validation_result
}

/// Get current configuration
pub async fn api_get_config(
    State(_app_state): State<AppState>,
) -> Result<Json<ApiResponse<ConfigResponse>>, ClewdrError> {
    let config = CLEWDR_CONFIG.load();

    let response = ConfigResponse {
        ban_config: config.ban.clone(),
        server_config: ServerConfig {
            ip: config.address().ip().to_string(),
            port: config.address().port(),
        },
        network_config: NetworkConfig {
            proxy: config.proxy.clone(),
            admin_password: "[REDACTED]".to_string(), // Never expose actual password
        },
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Update configuration
pub async fn api_update_config(
    State(app_state): State<AppState>,
    Json(request): Json<ConfigUpdateRequest>,
) -> Result<Json<ApiResponse<ConfigResponse>>, ClewdrError> {
    let config_arc = CLEWDR_CONFIG.load().clone();
    let mut config = (*config_arc).clone();

    // Update ban configuration
    if let Some(ban_config) = request.ban_config {
        config.ban = ban_config;
    }

    // Update server configuration
    if let Some(server_config) = request.server_config {
        use std::net::IpAddr;
        let ip: IpAddr = server_config
            .ip
            .parse()
            .map_err(|e| ClewdrError::BadRequest {
                msg: format!("Invalid IP address: {}", e),
            })?;
        config.set_ip(ip);
        config.set_port(server_config.port);
    }

    // Update network configuration
    if let Some(network_config) = request.network_config {
        config.proxy = network_config.proxy;
        // Only update password if it's not the redacted placeholder
        if network_config.admin_password != "[REDACTED]" {
            config.set_admin_password(network_config.admin_password)?;
        }
    }

    // Validate and save the configuration
    config = config.validate().map_err(|e| ClewdrError::BadRequest {
        msg: format!("Configuration validation failed: {}", e),
    })?;
    // Validate final ban/server configuration before saving
    let current_server = ServerConfig {
        ip: config.address().ip().to_string(),
        port: config.address().port(),
    };
    let validation = validate_ban_and_server(Some(&config.ban), Some(&current_server));
    if !validation.valid {
        let msg = if validation.errors.is_empty() {
            "配置不合法".to_string()
        } else {
            validation.errors.join("; ")
        };
        return Err(ClewdrError::BadRequest { msg });
    }
    config.save().await.map_err(|e| ClewdrError::Whatever {
        message: format!("Failed to save configuration: {}", e),
        source: Some(Box::new(e)),
    })?;

    // Update the global configuration
    CLEWDR_CONFIG.store(Arc::new(config.clone()));

    app_state
        .ban_farm
        .reload_config(config.ban.clone())
        .await
        .map_err(|e| ClewdrError::Whatever {
            message: format!("Failed to reload ban config: {}", e),
            source: Some(Box::new(e)),
        })?;

    let response = ConfigResponse {
        ban_config: config.ban.clone(),
        server_config: ServerConfig {
            ip: config.address().ip().to_string(),
            port: config.address().port(),
        },
        network_config: NetworkConfig {
            proxy: config.proxy.clone(),
            admin_password: "[REDACTED]".to_string(),
        },
    };

    audit_log(
        "config_update",
        None,
        &format!(
            "concurrency={} pause_seconds={} models={}",
            response.ban_config.concurrency,
            response.ban_config.pause_seconds,
            response.ban_config.models.join(",")
        ),
    );

    Ok(Json(ApiResponse::success(response)))
}

/// Reset configuration to defaults
pub async fn api_reset_config(
    State(app_state): State<AppState>,
) -> Result<Json<ApiResponse<ConfigResponse>>, ClewdrError> {
    let mut config = crate::config::ClewdrConfig::default();
    config = config
        .validate()
        .map_err(|e| ClewdrError::InternalServerError {
            msg: format!("Failed to reset configuration: {}", e),
        })?;
    config.save().await.map_err(|e| ClewdrError::Whatever {
        message: format!("Failed to save configuration: {}", e),
        source: Some(Box::new(e)),
    })?;

    // Update the global configuration
    CLEWDR_CONFIG.store(Arc::new(config.clone()));

    app_state
        .ban_farm
        .reload_config(config.ban.clone())
        .await
        .map_err(|e| ClewdrError::Whatever {
            message: format!("Failed to reload ban config: {}", e),
            source: Some(Box::new(e)),
        })?;

    let response = ConfigResponse {
        ban_config: config.ban.clone(),
        server_config: ServerConfig {
            ip: config.address().ip().to_string(),
            port: config.address().port(),
        },
        network_config: NetworkConfig {
            proxy: config.proxy.clone(),
            admin_password: "[REDACTED]".to_string(),
        },
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Get configuration validation status
pub async fn api_validate_config(
    State(_app_state): State<AppState>,
    Json(request): Json<ConfigUpdateRequest>,
) -> Result<Json<ApiResponse<ValidationResult>>, ClewdrError> {
    let validation_result =
        validate_ban_and_server(request.ban_config.as_ref(), request.server_config.as_ref());

    Ok(Json(ApiResponse::success(validation_result)))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_ban_config_rejects_zero_concurrency() {
        let ban = BanConfig {
            concurrency: 0,
            pause_seconds: 60,
            prompts_dir: "./ban_prompts".to_string(),
            models: vec!["claude-3-5-haiku-20241022".to_string()],
            max_tokens: 2048,
            request_timeout: 30_000,
        };
        let result = validate_ban_and_server(Some(&ban), None);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("并发数必须大于0")));
    }

    #[test]
    fn validate_ban_config_rejects_empty_models() {
        let ban = BanConfig {
            concurrency: 10,
            pause_seconds: 60,
            prompts_dir: "./ban_prompts".to_string(),
            models: Vec::new(),
            max_tokens: 2048,
            request_timeout: 30_000,
        };
        let result = validate_ban_and_server(Some(&ban), None);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("至少配置一个模型")));
    }
}

/// Export configuration
pub async fn api_export_config(
    State(_app_state): State<AppState>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ClewdrError> {
    let config = CLEWDR_CONFIG.load();

    // Create exportable config (without sensitive data)
    let export_config = serde_json::json!({
        "ban": config.ban,
        "server": {
            "ip": config.address().ip().to_string(),
            "port": config.address().port(),
        },
        "network": {
            "proxy": config.proxy,
        },
        "exported_at": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION"),
    });

    Ok(Json(ApiResponse::success(export_config)))
}

/// Import configuration
pub async fn api_import_config(
    State(app_state): State<AppState>,
    Json(import_data): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<ConfigResponse>>, ClewdrError> {
    // Parse and validate imported configuration
    let ban_config: Option<BanConfig> = import_data
        .get("ban")
        .and_then(|v| serde_json::from_value(v.clone()).ok());

    let server_config: Option<ServerConfig> = import_data
        .get("server")
        .and_then(|v| serde_json::from_value(v.clone()).ok());

    let network_config: Option<NetworkConfig> = import_data
        .get("network")
        .and_then(|v| serde_json::from_value(v.clone()).ok());

    let update_request = ConfigUpdateRequest {
        ban_config,
        server_config,
        network_config,
    };

    // Use the existing update endpoint
    api_update_config(State(app_state), Json(update_request)).await
}

/// Get configuration template for different scenarios
pub async fn api_get_config_template(
    State(_app_state): State<AppState>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ClewdrError> {
    let templates = serde_json::json!({
        "aggressive": {
            "name": "激进模式",
            "description": "最高并发、最短暂停时间，追求最大封号效率",
            "config": {
                "concurrency": 20,
                "pause_seconds": 30,
                "max_tokens": 2048,
                "models": ["claude-3-5-haiku-20241022"]
            }
        },
        "stealth": {
            "name": "隐蔽模式",
            "description": "低并发、长暂停时间，尽量避免触发限制",
            "config": {
                "concurrency": 1,
                "pause_seconds": 600,
                "max_tokens": 512,
                "models": ["claude-3-5-haiku-20241022"]
            }
        },
        "balanced": {
            "name": "平衡模式",
            "description": "中等并发与暂停时间，在稳定性和速度之间折中",
            "config": {
                "concurrency": 5,
                "pause_seconds": 120,
                "max_tokens": 1024,
                "models": ["claude-3-5-haiku-20241022", "claude-3-7-sonnet-20250219"]
            }
        }
    });

    Ok(Json(ApiResponse::success(templates)))
}
