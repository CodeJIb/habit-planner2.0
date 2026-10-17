use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use super::gamification::XpOutcome;

/// A JARVIS chat thread (`ai_conversations`).
#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AiConversation {
    pub id: i64,
    pub title: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// One persisted chat line (`ai_messages`). Roles are "user" or "assistant";
/// tool exchanges stay in memory and are never stored.
#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AiMessage {
    pub id: i64,
    pub conversation_id: i64,
    pub role: String,
    pub content: String,
    pub created_at: String,
}

/// What the frontend sends to `ai_chat`. `conversation_id` is null for a new thread.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatInput {
    #[serde(default)]
    pub conversation_id: Option<i64>,
    pub message: String,
}

/// Result of one chat turn: the thread, the newly persisted lines
/// (user + assistant) and any XP the assistant's actions earned.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatResult {
    pub conversation: AiConversation,
    pub messages: Vec<AiMessage>,
    pub xp: XpOutcome,
}

/// Assistant settings as the UI may see them. The API key itself never leaves Rust.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiSettingsView {
    pub api_key_set: bool,
    /// Empty means the built-in Anthropic provider; otherwise an OpenAI-compatible
    /// base URL (e.g. `http://localhost:11434/v1` for Ollama).
    pub base_url: String,
    pub model: String,
}

/// `api_key` / `base_url` / `model` of null leave the stored value untouched;
/// an empty string clears it (and resets the model to the default).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiSettingsInput {
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
}
