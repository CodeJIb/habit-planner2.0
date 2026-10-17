use tauri::State;

use crate::errors::AppError;
use crate::models::scheduled_block::{ScheduledBlock, ScheduledBlockInput};
use crate::services::scheduled_block::ScheduledBlockService;
use crate::state::AppState;

/// All blocks, or just those overlapping "YYYY-MM-DD" bounds when both are given.
#[tauri::command]
pub async fn list_blocks(
    state: State<'_, AppState>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<Vec<ScheduledBlock>, AppError> {
    ScheduledBlockService::new(state.db.clone())
        .list(start_date, end_date)
        .await
}

#[tauri::command]
pub async fn create_block(
    state: State<'_, AppState>,
    input: ScheduledBlockInput,
) -> Result<ScheduledBlock, AppError> {
    ScheduledBlockService::new(state.db.clone()).create(input).await
}

#[tauri::command]
pub async fn update_block(
    state: State<'_, AppState>,
    id: i64,
    input: ScheduledBlockInput,
) -> Result<ScheduledBlock, AppError> {
    ScheduledBlockService::new(state.db.clone()).update(id, input).await
}

#[tauri::command]
pub async fn set_block_status(
    state: State<'_, AppState>,
    id: i64,
    status: String,
) -> Result<ScheduledBlock, AppError> {
    ScheduledBlockService::new(state.db.clone()).set_status(id, status).await
}

#[tauri::command]
pub async fn delete_block(state: State<'_, AppState>, id: i64) -> Result<(), AppError> {
    ScheduledBlockService::new(state.db.clone()).delete(id).await
}
