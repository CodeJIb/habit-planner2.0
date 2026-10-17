use tauri::State;

use crate::errors::AppError;
use crate::models::habit::{Habit, HabitInput, HabitToggleResult, HabitView};
use crate::services::habit::HabitService;
use crate::state::AppState;

#[tauri::command]
pub async fn create_habit(state: State<'_, AppState>, input: HabitInput) -> Result<Habit, AppError> {
    HabitService::new(state.db.clone()).create_habit(input).await
}

/// `today` is the user's local date as "YYYY-MM-DD".
#[tauri::command]
pub async fn list_habits(state: State<'_, AppState>, today: String) -> Result<Vec<HabitView>, AppError> {
    HabitService::new(state.db.clone()).list_habits(&today).await
}

#[tauri::command]
pub async fn update_habit(state: State<'_, AppState>, id: i64, input: HabitInput) -> Result<Habit, AppError> {
    HabitService::new(state.db.clone()).update_habit(id, input).await
}

#[tauri::command]
pub async fn delete_habit(state: State<'_, AppState>, id: i64) -> Result<(), AppError> {
    HabitService::new(state.db.clone()).delete_habit(id).await
}

#[tauri::command]
pub async fn toggle_habit_completion(
    state: State<'_, AppState>,
    habit_id: i64,
    local_date: String,
) -> Result<HabitToggleResult, AppError> {
    HabitService::new(state.db.clone())
        .toggle_completion(habit_id, &local_date)
        .await
}
