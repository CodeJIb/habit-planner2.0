use tauri::State;

use crate::errors::AppError;
use crate::models::focus_session::{FocusResult, FocusSession, FocusSessionInput, FocusStats};
use crate::services::focus_session::FocusSessionService;
use crate::state::AppState;

/// Recent focus sessions, optionally only those linked to `task_id`.
#[tauri::command]
pub async fn list_focus_sessions(
    state: State<'_, AppState>,
    task_id: Option<i64>,
    limit: Option<i64>,
) -> Result<Vec<FocusSession>, AppError> {
    FocusSessionService::new(state.db.clone())
        .list(task_id, limit)
        .await
}

#[tauri::command]
pub async fn start_focus_session(
    state: State<'_, AppState>,
    input: FocusSessionInput,
) -> Result<FocusSession, AppError> {
    FocusSessionService::new(state.db.clone()).start(input).await
}

#[tauri::command]
pub async fn complete_focus_session(
    state: State<'_, AppState>,
    id: i64,
    actual_duration_seconds: i64,
) -> Result<FocusResult, AppError> {
    FocusSessionService::new(state.db.clone())
        .complete(id, actual_duration_seconds)
        .await
}

#[tauri::command]
pub async fn cancel_focus_session(
    state: State<'_, AppState>,
    id: i64,
) -> Result<FocusResult, AppError> {
    FocusSessionService::new(state.db.clone()).cancel(id).await
}

#[tauri::command]
pub async fn delete_focus_session(state: State<'_, AppState>, id: i64) -> Result<(), AppError> {
    FocusSessionService::new(state.db.clone()).delete(id).await
}

#[tauri::command]
pub async fn focus_stats(state: State<'_, AppState>) -> Result<FocusStats, AppError> {
    FocusSessionService::new(state.db.clone()).stats().await
}
