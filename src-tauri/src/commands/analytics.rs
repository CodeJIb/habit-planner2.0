use tauri::State;

use crate::errors::AppError;
use crate::models::analytics::{AnalyticsSummary, DayCount, DayMinutes, XpPoint};
use crate::services::analytics::AnalyticsService;
use crate::state::AppState;

/// Cumulative XP per day. With no `days` window the whole ledger is returned.
#[tauri::command]
pub async fn list_xp_over_time(
    state: State<'_, AppState>,
    days: Option<i64>,
) -> Result<Vec<XpPoint>, AppError> {
    AnalyticsService::new(state.db.clone()).xp_over_time(days).await
}

#[tauri::command]
pub async fn list_task_completions(
    state: State<'_, AppState>,
    days: Option<i64>,
) -> Result<Vec<DayCount>, AppError> {
    AnalyticsService::new(state.db.clone()).task_completions(days).await
}

#[tauri::command]
pub async fn list_focus_minutes(
    state: State<'_, AppState>,
    days: Option<i64>,
) -> Result<Vec<DayMinutes>, AppError> {
    AnalyticsService::new(state.db.clone()).focus_minutes(days).await
}

/// Every local date with a habit check-in, for the consistency heat map.
#[tauri::command]
pub async fn list_habit_checkins(state: State<'_, AppState>) -> Result<Vec<String>, AppError> {
    AnalyticsService::new(state.db.clone()).habit_checkins().await
}

#[tauri::command]
pub async fn list_analytics_summary(
    state: State<'_, AppState>,
) -> Result<AnalyticsSummary, AppError> {
    AnalyticsService::new(state.db.clone()).summary().await
}
