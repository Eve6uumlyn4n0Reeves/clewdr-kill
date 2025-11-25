use axum::{Json, extract::State};
use axum_auth::AuthBearer;
use serde::{Deserialize, Serialize};

use super::error::ApiError;
use crate::{
    config::{BanConfig, CLEWDR_CONFIG},
    router::AppState,
};
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

/// Get current configuration
pub async fn api_get_config(
    State(_app_state): State<AppState>,
    AuthBearer(token): AuthBearer,
) -> Result<Json<ConfigResponse>, ApiError> {
    if !CLEWDR_CONFIG.load().admin_auth(&token) {
        return Err(ApiError::unauthorized());
    }

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

    Ok(Json(response))
}

/// Update configuration
pub async fn api_update_config(
    State(_app_state): State<AppState>,
    AuthBearer(token): AuthBearer,
    Json(request): Json<ConfigUpdateRequest>,
) -> Result<Json<ConfigResponse>, ApiError> {
    if !CLEWDR_CONFIG.load().admin_auth(&token) {
        return Err(ApiError::unauthorized());
    }

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
            .map_err(|e| ApiError::bad_request(&format!("Invalid IP address: {}", e)))?;
        config.set_ip(ip);
        config.set_port(server_config.port);
    }

    // Update network configuration
    if let Some(network_config) = request.network_config {
        config.proxy = network_config.proxy;
        // Only update password if it's not the redacted placeholder
        if network_config.admin_password != "[REDACTED]" {
            config.set_admin_password(network_config.admin_password);
        }
    }

    // Validate and save the configuration
    config = config.validate();
    config
        .save()
        .await
        .map_err(|e| ApiError::internal(&format!("Failed to save configuration: {}", e)))?;

    // Update the global configuration
    CLEWDR_CONFIG.store(Arc::new(config.clone()));

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

    Ok(Json(response))
}

/// Reset configuration to defaults
pub async fn api_reset_config(
    State(_app_state): State<AppState>,
    AuthBearer(token): AuthBearer,
) -> Result<Json<ConfigResponse>, ApiError> {
    if !CLEWDR_CONFIG.load().admin_auth(&token) {
        return Err(ApiError::unauthorized());
    }

    let mut config = crate::config::ClewdrConfig::default();
    config = config.validate();
    config
        .save()
        .await
        .map_err(|e| ApiError::internal(&format!("Failed to save configuration: {}", e)))?;

    // Update the global configuration
    CLEWDR_CONFIG.store(Arc::new(config.clone()));

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

    Ok(Json(response))
}

/// Get configuration validation status
pub async fn api_validate_config(
    State(_app_state): State<AppState>,
    AuthBearer(token): AuthBearer,
    Json(request): Json<ConfigUpdateRequest>,
) -> Result<Json<ValidationResult>, ApiError> {
    if !CLEWDR_CONFIG.load().admin_auth(&token) {
        return Err(ApiError::unauthorized());
    }

    let mut validation_result = ValidationResult {
        valid: true,
        errors: Vec::new(),
        warnings: Vec::new(),
    };

    // Validate ban configuration
    if let Some(ban_config) = &request.ban_config {
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

        if ban_config.request_jitter_min >= ban_config.request_jitter_max {
            validation_result.valid = false;
            validation_result
                .errors
                .push("最小延迟必须小于最大延迟".to_string());
        }

        if ban_config.models.is_empty() {
            validation_result.valid = false;
            validation_result
                .errors
                .push("必须至少配置一个模型".to_string());
        }
    }

    // Validate server configuration
    if let Some(server_config) = &request.server_config {
        if server_config.port == 0 {
            validation_result.valid = false;
            validation_result.errors.push("无效的端口号".to_string());
        }

        if server_config.ip.parse::<std::net::IpAddr>().is_err() {
            validation_result.valid = false;
            validation_result.errors.push("无效的IP地址".to_string());
        }
    }

    // Validate working hours
    if let Some(ban_config) = &request.ban_config {
        if ban_config.working_hours.enabled {
            // Validate time format
            if chrono::NaiveTime::parse_from_str(&ban_config.working_hours.start, "%H:%M").is_err()
            {
                validation_result.valid = false;
                validation_result
                    .errors
                    .push("开始时间格式无效，应为 HH:MM".to_string());
            }

            if chrono::NaiveTime::parse_from_str(&ban_config.working_hours.end, "%H:%M").is_err() {
                validation_result.valid = false;
                validation_result
                    .errors
                    .push("结束时间格式无效，应为 HH:MM".to_string());
            }
        }
    }

    Ok(Json(validation_result))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Export configuration
pub async fn api_export_config(
    State(_app_state): State<AppState>,
    AuthBearer(token): AuthBearer,
) -> Result<Json<serde_json::Value>, ApiError> {
    if !CLEWDR_CONFIG.load().admin_auth(&token) {
        return Err(ApiError::unauthorized());
    }

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

    Ok(Json(export_config))
}

/// Import configuration
pub async fn api_import_config(
    State(_app_state): State<AppState>,
    AuthBearer(token): AuthBearer,
    Json(import_data): Json<serde_json::Value>,
) -> Result<Json<ConfigResponse>, ApiError> {
    if !CLEWDR_CONFIG.load().admin_auth(&token) {
        return Err(ApiError::unauthorized());
    }

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
    api_update_config(State(_app_state), AuthBearer(token), Json(update_request)).await
}

/// Get configuration template for different scenarios
pub async fn api_get_config_template(
    State(_app_state): State<AppState>,
    AuthBearer(token): AuthBearer,
) -> Result<Json<serde_json::Value>, ApiError> {
    if !CLEWDR_CONFIG.load().admin_auth(&token) {
        return Err(ApiError::unauthorized());
    }

    let templates = serde_json::json!({
        "aggressive": {
            "ban": {
                "concurrency": 10,
                "pause_seconds": 60,
                "max_tokens": 1024,
                "adaptive_throttling": false,
                "smart_error_handling": false,
                "request_jitter_min": 50,
                "request_jitter_max": 200,
                "retry_attempts": 5,
                "working_hours": {
                    "enabled": false
                }
            }
        },
        "stealth": {
            "ban": {
                "concurrency": 1,
                "pause_seconds": 600,
                "max_tokens": 256,
                "adaptive_throttling": true,
                "smart_error_handling": true,
                "user_agent_rotation": true,
                "proxy_rotation": true,
                "request_jitter_min": 1000,
                "request_jitter_max": 5000,
                "retry_attempts": 1,
                "working_hours": {
                    "enabled": true,
                    "start": "22:00",
                    "end": "06:00",
                    "timezone": "UTC"
                }
            }
        },
        "balanced": {
            "ban": {
                "concurrency": 3,
                "pause_seconds": 180,
                "max_tokens": 512,
                "adaptive_throttling": true,
                "smart_error_handling": true,
                "request_jitter_min": 300,
                "request_jitter_max": 1200,
                "retry_attempts": 2,
                "working_hours": {
                    "enabled": true,
                    "start": "09:00",
                    "end": "18:00",
                    "timezone": "UTC"
                }
            }
        }
    });

    Ok(Json(templates))
}
