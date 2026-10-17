use tauri::State;

use crate::errors::AppError;
use crate::models::gamification::PlayerProfileView;
use crate::services::gamification;
use crate::state::AppState;

#[tauri::command]
pub async fn get_player_profile(state: State<'_, AppState>) -> Result<PlayerProfileView, AppError> {
    let mut conn = state.db.acquire().await?;
    gamification::get_profile(&mut conn).await
}
