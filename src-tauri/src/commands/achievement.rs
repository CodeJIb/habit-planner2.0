use tauri::State;

use crate::errors::AppError;
use crate::models::achievement::AchievementView;
use crate::services::achievement;
use crate::state::AppState;

/// Every achievement with the player's unlock state.
#[tauri::command]
pub async fn list_achievements(
    state: State<'_, AppState>,
) -> Result<Vec<AchievementView>, AppError> {
    let mut conn = state.db.acquire().await?;
    achievement::list(&mut conn).await
}
