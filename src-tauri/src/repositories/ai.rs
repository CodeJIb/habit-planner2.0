use sqlx::SqliteConnection;

use crate::errors::AppError;
use crate::models::ai::{AiConversation, AiMessage};

// SQL fragment for "now" in the same ISO format the schema uses for defaults.
const NOW: &str = "strftime('%Y-%m-%dT%H:%M:%SZ','now')";

pub async fn insert_conversation(
    conn: &mut SqliteConnection,
    title: Option<&str>,
) -> Result<AiConversation, AppError> {
    let conversation =
        sqlx::query_as::<_, AiConversation>("INSERT INTO ai_conversations (title) VALUES (?) RETURNING *")
            .bind(title)
            .fetch_one(&mut *conn)
            .await?;
    Ok(conversation)
}

pub async fn find_conversation(
    conn: &mut SqliteConnection,
    id: i64,
) -> Result<Option<AiConversation>, AppError> {
    let conversation =
        sqlx::query_as::<_, AiConversation>("SELECT * FROM ai_conversations WHERE id = ?")
            .bind(id)
            .fetch_optional(&mut *conn)
            .await?;
    Ok(conversation)
}

/// Most recently active threads first.
pub async fn list_conversations(
    conn: &mut SqliteConnection,
    limit: i64,
) -> Result<Vec<AiConversation>, AppError> {
    let conversations = sqlx::query_as::<_, AiConversation>(
        "SELECT * FROM ai_conversations ORDER BY updated_at DESC, id DESC LIMIT ?",
    )
    .bind(limit)
    .fetch_all(&mut *conn)
    .await?;
    Ok(conversations)
}

/// Bump `updated_at` after a new message and give the thread its title on first use
/// (`COALESCE` keeps a title that is already set).
pub async fn touch_conversation(
    conn: &mut SqliteConnection,
    id: i64,
    title: Option<&str>,
) -> Result<AiConversation, AppError> {
    let sql = format!(
        "UPDATE ai_conversations SET updated_at = {NOW}, title = COALESCE(title, ?1) WHERE id = ?2 RETURNING *"
    );
    let conversation = sqlx::query_as::<_, AiConversation>(&sql)
        .bind(title)
        .bind(id)
        .fetch_optional(&mut *conn)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(conversation)
}

pub async fn delete_conversation(conn: &mut SqliteConnection, id: i64) -> Result<bool, AppError> {
    // ai_messages cascade with the thread (ON DELETE CASCADE in the schema).
    let result = sqlx::query("DELETE FROM ai_conversations WHERE id = ?")
        .bind(id)
        .execute(&mut *conn)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn insert_message(
    conn: &mut SqliteConnection,
    conversation_id: i64,
    role: &str,
    content: &str,
) -> Result<AiMessage, AppError> {
    let message = sqlx::query_as::<_, AiMessage>(
        "INSERT INTO ai_messages (conversation_id, role, content) VALUES (?, ?, ?) RETURNING *",
    )
    .bind(conversation_id)
    .bind(role)
    .bind(content)
    .fetch_one(&mut *conn)
    .await?;
    Ok(message)
}

/// The conversation's messages oldest-first, capped at the newest `limit` rows.
pub async fn list_messages(
    conn: &mut SqliteConnection,
    conversation_id: i64,
    limit: i64,
) -> Result<Vec<AiMessage>, AppError> {
    let messages = sqlx::query_as::<_, AiMessage>(
        "SELECT * FROM (
             SELECT * FROM ai_messages WHERE conversation_id = ? ORDER BY id DESC LIMIT ?
         ) ORDER BY id ASC",
    )
    .bind(conversation_id)
    .bind(limit)
    .fetch_all(&mut *conn)
    .await?;
    Ok(messages)
}

// The key/value helpers live in `repositories::settings`; re-exported here so
// existing callers (`repositories::ai::get_setting`, ...) keep working.
pub use super::settings::{delete_setting, get_setting, set_setting};
