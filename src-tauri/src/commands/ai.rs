use tauri::{Emitter, State};

use crate::errors::AppError;
use crate::models::ai::{
    AiConversation, AiMessage, AiSettingsInput, AiSettingsView, ChatInput, ChatResult,
};
use crate::services::ai::AiService;
use crate::state::AppState;

/// Event name carrying streamed assistant text to the frontend.
pub const AI_DELTA_EVENT: &str = "ai:delta";

#[tauri::command]
pub async fn ai_chat(state: State<'_, AppState>, request: ChatInput) -> Result<ChatResult, AppError> {
    AiService::new(state.db.clone())
        .with_app(state.app.clone())
        .chat(request)
        .await
}

/// Like `ai_chat`, but emits `ai:delta` events with the assistant's words as
/// they arrive so the UI can show progress on a slow connection.
#[tauri::command]
pub async fn ai_chat_stream(
    state: State<'_, AppState>,
    request: ChatInput,
) -> Result<ChatResult, AppError> {
    let app = state.app.clone();
    AiService::new(state.db.clone())
        .with_app(state.app.clone())
        .chat_streaming(request, move |delta| {
            let _ = app.emit(AI_DELTA_EVENT, delta.to_string());
        })
        .await
}

#[tauri::command]
pub async fn list_conversations(
    state: State<'_, AppState>,
) -> Result<Vec<AiConversation>, AppError> {
    AiService::new(state.db.clone()).list_conversations().await
}

#[tauri::command]
pub async fn get_conversation(
    state: State<'_, AppState>,
    id: i64,
) -> Result<Vec<AiMessage>, AppError> {
    AiService::new(state.db.clone()).get_conversation(id).await
}

#[tauri::command]
pub async fn delete_conversation(state: State<'_, AppState>, id: i64) -> Result<(), AppError> {
    AiService::new(state.db.clone()).delete_conversation(id).await
}

#[tauri::command]
pub async fn get_ai_settings(state: State<'_, AppState>) -> Result<AiSettingsView, AppError> {
    AiService::new(state.db.clone())
        .with_app(state.app.clone())
        .get_settings()
        .await
}

#[tauri::command]
pub async fn set_ai_settings(
    state: State<'_, AppState>,
    input: AiSettingsInput,
) -> Result<AiSettingsView, AppError> {
    AiService::new(state.db.clone())
        .with_app(state.app.clone())
        .set_settings(input)
        .await
}
