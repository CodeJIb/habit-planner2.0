use tauri::State;

use crate::errors::AppError;
use crate::models::task::{Task, TaskInput, TaskStatusResult};
use crate::services::task::TaskService;
use crate::state::AppState;

#[tauri::command]
pub async fn create_task(state: State<'_, AppState>, input: TaskInput) -> Result<Task, AppError> {
    TaskService::new(state.db.clone()).create_task(input).await
}

#[tauri::command]
pub async fn get_task(state: State<'_, AppState>, id: i64) -> Result<Task, AppError> {
    TaskService::new(state.db.clone()).get_task(id).await
}

#[tauri::command]
pub async fn list_tasks(state: State<'_, AppState>, status: Option<String>) -> Result<Vec<Task>, AppError> {
    TaskService::new(state.db.clone()).list_tasks(status).await
}

#[tauri::command]
pub async fn update_task(state: State<'_, AppState>, id: i64, input: TaskInput) -> Result<Task, AppError> {
    TaskService::new(state.db.clone()).update_task(id, input).await
}

#[tauri::command]
pub async fn set_task_status(
    state: State<'_, AppState>,
    id: i64,
    status: String,
) -> Result<TaskStatusResult, AppError> {
    TaskService::new(state.db.clone()).set_status(id, status).await
}

#[tauri::command]
pub async fn delete_task(state: State<'_, AppState>, id: i64) -> Result<(), AppError> {
    TaskService::new(state.db.clone()).delete_task(id).await
}
