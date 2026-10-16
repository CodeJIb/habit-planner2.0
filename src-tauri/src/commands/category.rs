use tauri::State;

use crate::errors::AppError;
use crate::models::category::{Category, CategoryInput};
use crate::services::category::CategoryService;
use crate::state::AppState;

#[tauri::command]
pub async fn list_categories(state: State<'_, AppState>) -> Result<Vec<Category>, AppError> {
    CategoryService::new(state.db.clone()).list().await
}

#[tauri::command]
pub async fn create_category(state: State<'_, AppState>, input: CategoryInput) -> Result<Category, AppError> {
    CategoryService::new(state.db.clone()).create(input).await
}

#[tauri::command]
pub async fn delete_category(state: State<'_, AppState>, id: i64) -> Result<(), AppError> {
    CategoryService::new(state.db.clone()).delete(id).await
}
