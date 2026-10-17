use tauri::State;

use crate::errors::AppError;
use crate::models::goal::{GoalInput, GoalStatusResult, GoalView, MilestoneInput};
use crate::services::goal::GoalService;
use crate::state::AppState;

#[tauri::command]
pub async fn list_goals(
    state: State<'_, AppState>,
    status: Option<String>,
) -> Result<Vec<GoalView>, AppError> {
    GoalService::new(state.db.clone()).list_goals(status).await
}

#[tauri::command]
pub async fn create_goal(
    state: State<'_, AppState>,
    input: GoalInput,
) -> Result<GoalView, AppError> {
    GoalService::new(state.db.clone()).create_goal(input).await
}

#[tauri::command]
pub async fn update_goal(
    state: State<'_, AppState>,
    id: i64,
    input: GoalInput,
) -> Result<GoalView, AppError> {
    GoalService::new(state.db.clone()).update_goal(id, input).await
}

#[tauri::command]
pub async fn set_goal_status(
    state: State<'_, AppState>,
    id: i64,
    status: String,
) -> Result<GoalStatusResult, AppError> {
    GoalService::new(state.db.clone()).set_status(id, status).await
}

#[tauri::command]
pub async fn delete_goal(state: State<'_, AppState>, id: i64) -> Result<(), AppError> {
    GoalService::new(state.db.clone()).delete_goal(id).await
}

#[tauri::command]
pub async fn add_goal_milestone(
    state: State<'_, AppState>,
    goal_id: i64,
    input: MilestoneInput,
) -> Result<GoalView, AppError> {
    GoalService::new(state.db.clone()).add_milestone(goal_id, input).await
}

#[tauri::command]
pub async fn set_goal_milestone_status(
    state: State<'_, AppState>,
    id: i64,
    status: String,
) -> Result<GoalView, AppError> {
    GoalService::new(state.db.clone()).set_milestone_status(id, status).await
}

#[tauri::command]
pub async fn delete_goal_milestone(
    state: State<'_, AppState>,
    id: i64,
) -> Result<GoalView, AppError> {
    GoalService::new(state.db.clone()).delete_milestone(id).await
}
