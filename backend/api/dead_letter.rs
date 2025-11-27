use axum::{extract::State, Json};
use serde::Serialize;

use crate::{
    api::response::{success_response, ApiResponse, EmptyResponse},
    error::ClewdrError,
    router::AppState,
    services::dead_letter_queue::DeadLetterEntry,
};

#[derive(Serialize)]
pub struct DeadLetterStats {
    pub total_entries: usize,
    pub entries: Vec<DeadLetterEntry>,
}

/// 获取死信队列内容
pub async fn api_get_dead_letters(
    State(app_state): State<AppState>,
) -> Result<Json<ApiResponse<DeadLetterStats>>, ClewdrError> {
    let queue = app_state.ban_queue_handle;
    let dlq = queue.dead_letter_queue();

    let entries = dlq.get_all().await;
    let total_entries = entries.len();

    Ok(Json(ApiResponse::success(DeadLetterStats {
        total_entries,
        entries,
    })))
}

/// 清空死信队列
pub async fn api_clear_dead_letters(
    State(app_state): State<AppState>,
) -> Result<Json<EmptyResponse>, ClewdrError> {
    let queue = app_state.ban_queue_handle;
    let dlq = queue.dead_letter_queue();

    dlq.clear().await;

    Ok(Json(success_response()))
}
