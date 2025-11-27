use crate::api::response::{success_response, ApiResponse, EmptyResponse};
use crate::error::ClewdrError;
use crate::middleware::{validate_path, validate_prompt_content, validate_prompt_name};
use crate::{config::CLEWDR_CONFIG, router::AppState};
use crate::utils::logging::audit_log;
use axum::{extract::State, Json};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, time::SystemTime};
use tokio::fs;
use tracing::{error, info};

#[derive(Debug, Serialize, Deserialize)]
pub struct PromptFile {
    pub name: String,
    pub content: String,
    pub created_at: String,
    pub modified_at: String,
    pub size: usize,
}

/// Get all prompt files
pub async fn api_get_prompts(
    State(_app_state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<PromptFile>>>, ClewdrError> {
    let prompts_dir = CLEWDR_CONFIG.load().ban.prompts_dir.clone();
    let validated_dir = validate_path(&prompts_dir)?;
    let prompts_path = PathBuf::from(&validated_dir);

    if !prompts_path.exists() {
        return Err(ClewdrError::PathNotFound {
            msg: format!("Prompt directory not found: {}", prompts_dir),
        });
    }

    let mut entries = match fs::read_dir(&prompts_path).await {
        Ok(e) => e,
        Err(e) => {
            error!("Error reading prompts directory: {}", e);
            return Err(ClewdrError::Whatever {
                message: "Failed to read prompts directory".to_string(),
                source: Some(Box::new(e)),
            });
        }
    };

    let mut prompts = Vec::new();

    loop {
        match entries.next_entry().await {
            Ok(Some(entry)) => {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("txt") {
                    match fs::read_to_string(&path).await {
                        Ok(content) => {
                            let metadata = match fs::metadata(&path).await {
                                Ok(m) => m,
                                Err(e) => {
                                    error!("Error reading metadata for {:?}: {}", path, e);
                                    continue;
                                }
                            };

                            let created_at = metadata_created_at(&metadata);
                            let modified_at = metadata_modified_at(&metadata);

                            let name = path
                                .file_stem()
                                .and_then(|n| n.to_str())
                                .unwrap_or("unknown")
                                .to_string();

                            let content_cloned = content.clone();
                            prompts.push(PromptFile {
                                name,
                                content: content_cloned,
                                created_at,
                                modified_at,
                                size: content.len(),
                            });
                        }
                        Err(e) => {
                            error!("Error reading file {:?}: {}", path, e);
                        }
                    }
                }
            }
            Ok(None) => {
                // No more entries
                break;
            }
            Err(e) => {
                error!("Error reading directory entry: {}", e);
                break;
            }
        }
    }

    prompts.sort_by(|a, b| a.name.cmp(&b.name));

    if prompts.is_empty() {
        return Err(ClewdrError::PathNotFound {
            msg: format!(
                "No prompt files found in directory: {}. Please create or upload prompts.",
                prompts_dir
            ),
        });
    }

    Ok(Json(ApiResponse::success(prompts)))
}

/// Get a specific prompt file
pub async fn api_get_prompt(
    State(_app_state): State<AppState>,
    Json(payload): Json<GetPromptRequest>,
) -> Result<Json<ApiResponse<PromptFile>>, ClewdrError> {
    let prompts_dir = CLEWDR_CONFIG.load().ban.prompts_dir.clone();
    let validated_dir = validate_path(&prompts_dir)?;
    let validated_name = validate_prompt_name(&payload.name)?;
    let sanitized_name = sanitize_filename(&validated_name);
    let prompt_path = PathBuf::from(&validated_dir).join(format!("{}.txt", sanitized_name));

    if !prompt_path.exists() {
        return Err(ClewdrError::PathNotFound {
            msg: format!("Prompt '{}' not found", payload.name),
        });
    }

    match fs::read_to_string(&prompt_path).await {
        Ok(content) => {
            let metadata = match fs::metadata(&prompt_path).await {
                Ok(m) => m,
                Err(e) => {
                    error!("Error reading metadata for {:?}: {}", prompt_path, e);
                    return Err(ClewdrError::Whatever {
                        message: "Failed to read prompt metadata".to_string(),
                        source: Some(Box::new(e)),
                    });
                }
            };

            let created_at = metadata_created_at(&metadata);
            let modified_at = metadata_modified_at(&metadata);

            let content_cloned = content.clone();
            Ok(Json(ApiResponse::success(PromptFile {
                name: payload.name.clone(),
                content: content_cloned,
                created_at,
                modified_at,
                size: content.len(),
            })))
        }
        Err(e) => {
            error!("Error reading prompt file: {}", e);
            Err(ClewdrError::Whatever {
                message: "Failed to read prompt file".to_string(),
                source: Some(Box::new(e)),
            })
        }
    }
}

/// Create or update a prompt file
pub async fn api_save_prompt(
    State(app_state): State<AppState>,
    Json(payload): Json<SavePromptRequest>,
) -> Result<Json<ApiResponse<PromptFile>>, ClewdrError> {
    let prompts_dir = CLEWDR_CONFIG.load().ban.prompts_dir.clone();
    let validated_dir = validate_path(&prompts_dir)?;
    let prompts_path = PathBuf::from(&validated_dir);

    // Create directory if it doesn't exist
    if !prompts_path.exists() {
        if let Err(e) = fs::create_dir_all(&prompts_path).await {
            error!("Error creating prompts directory: {}", e);
            return Err(ClewdrError::Whatever {
                message: "Failed to create prompts directory".to_string(),
                source: Some(Box::new(e)),
            });
        }
    }

    // Validate prompt name and content
    let validated_name = validate_prompt_name(&payload.name)?;
    let validated_content = validate_prompt_content(&payload.content)?;

    // Use additional filename sanitization
    let sanitized_name = sanitize_filename(&validated_name);
    let prompt_path = prompts_path.join(format!("{}.txt", sanitized_name));

    // Get metadata to check if file exists
    let created_at = if prompt_path.exists() {
        match fs::metadata(&prompt_path).await {
            Ok(meta) => metadata_created_at(&meta),
            Err(_) => Utc::now().to_rfc3339(),
        }
    } else {
        Utc::now().to_rfc3339()
    };

    match fs::write(&prompt_path, &validated_content).await {
        Ok(_) => {
            info!("Saved prompt: {}", sanitized_name);

            let modified_at = Utc::now().to_rfc3339();

            let content_len = payload.content.len();
            let response = PromptFile {
                name: sanitized_name,
                content: payload.content.clone(),
                created_at,
                modified_at,
                size: content_len,
            };

            // 重载提示词以恢复 worker
            if let Err(e) = app_state.ban_farm.reload_prompts().await {
                error!("Failed to reload prompts after save: {}", e);
            }

            audit_log(
                "prompt_save",
                None,
                &format!("name={} size={}", response.name, response.size),
            );

            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => {
            error!("Error saving prompt file: {}", e);
            Err(ClewdrError::Whatever {
                message: "Failed to save prompt file".to_string(),
                source: Some(Box::new(e)),
            })
        }
    }
}

/// Delete a prompt file
pub async fn api_delete_prompt(
    State(app_state): State<AppState>,
    Json(payload): Json<DeletePromptRequest>,
) -> Result<Json<EmptyResponse>, ClewdrError> {
    let prompts_dir = CLEWDR_CONFIG.load().ban.prompts_dir.clone();
    let validated_dir = validate_path(&prompts_dir)?;
    let validated_name = validate_prompt_name(&payload.name)?;
    let sanitized_name = sanitize_filename(&validated_name);
    let prompt_path = PathBuf::from(&validated_dir).join(format!("{}.txt", sanitized_name));

    if !prompt_path.exists() {
        return Err(ClewdrError::PathNotFound {
            msg: format!("Prompt '{}' not found", payload.name),
        });
    }

    match fs::remove_file(&prompt_path).await {
        Ok(_) => {
            info!("Deleted prompt: {}", payload.name);
            if let Err(e) = app_state.ban_farm.reload_prompts().await {
                error!("Failed to reload prompts after delete: {}", e);
            }
            audit_log("prompt_delete", None, &payload.name);
            Ok(Json(success_response()))
        }
        Err(e) => {
            error!("Error deleting prompt file: {}", e);
            Err(ClewdrError::Whatever {
                message: "Failed to delete prompt file".to_string(),
                source: Some(Box::new(e)),
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct GetPromptRequest {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct SavePromptRequest {
    pub name: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct DeletePromptRequest {
    pub name: String,
}

// Helper function to sanitize filename
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_lowercase()
}

fn metadata_created_at(metadata: &std::fs::Metadata) -> String {
    metadata
        .created()
        .or_else(|_| metadata.modified())
        .map(system_time_to_rfc3339)
        .unwrap_or_default()
}

fn metadata_modified_at(metadata: &std::fs::Metadata) -> String {
    metadata
        .modified()
        .map(system_time_to_rfc3339)
        .unwrap_or_default()
}

fn system_time_to_rfc3339(time: SystemTime) -> String {
    chrono::DateTime::<Utc>::from(time).to_rfc3339()
}
