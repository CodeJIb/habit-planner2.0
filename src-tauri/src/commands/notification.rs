use tauri::State;

use crate::errors::AppError;
use crate::models::notification::{NotificationSettings, NotificationSettingsInput, ReminderResult};
use crate::services::notification::NotificationService;
use crate::state::AppState;

#[tauri::command]
pub async fn get_notification_settings(
    state: State<'_, AppState>,
) -> Result<NotificationSettings, AppError> {
    NotificationService::new(state.db.clone()).get_settings().await
}

#[tauri::command]
pub async fn set_notification_settings(
    state: State<'_, AppState>,
    input: NotificationSettingsInput,
) -> Result<NotificationSettings, AppError> {
    NotificationService::new(state.db.clone())
        .set_settings(input)
        .await
}

#[tauri::command]
pub async fn send_test_notification(state: State<'_, AppState>) -> Result<(), AppError> {
    NotificationService::new(state.db.clone()).send_test().await
}

#[tauri::command]
pub async fn send_due_reminders(state: State<'_, AppState>) -> Result<ReminderResult, AppError> {
    NotificationService::new(state.db.clone()).run_due_check().await
}
