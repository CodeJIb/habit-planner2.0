use tauri::State;

use crate::errors::AppError;
use crate::models::quest::QuestView;
use crate::services::quest;
use crate::state::AppState;

/// Quests for the current day/week, with progress.
#[tauri::command]
pub async fn list_quests(state: State<'_, AppState>) -> Result<Vec<QuestView>, AppError> {
    let mut conn = state.db.acquire().await?;
    quest::list(&mut conn).await
}
