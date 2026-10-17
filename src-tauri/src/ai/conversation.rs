//! Phase 5: store and load conversations in ai_conversations / ai_messages.
//! Only user and assistant lines are persisted (the schema enforces those two
//! roles); tool exchanges live for the duration of one request.

use sqlx::SqliteConnection;

use crate::ai::provider::{ContentBlock, ProviderMessage};
use crate::errors::AppError;
use crate::models::ai::{AiConversation, AiMessage};
use crate::repositories::ai as repo;

const MAX_TITLE_CHARS: usize = 48;

/// The stored history (oldest first) shaped for the provider: one text block per turn.
pub fn to_provider_messages(history: &[AiMessage]) -> Vec<ProviderMessage> {
    to_provider_messages_capped(history, usize::MAX)
}

/// Like [`to_provider_messages`], but each line is truncated to `max_chars`.
/// Used on the live path so one huge pasted message cannot blow the token budget.
pub fn to_provider_messages_capped(
    history: &[AiMessage],
    max_chars: usize,
) -> Vec<ProviderMessage> {
    history
        .iter()
        .map(|message| {
            ProviderMessage::text(&message.role, &truncate(&message.content, max_chars))
        })
        .collect()
}

/// Cut `text` to at most `max` characters (not bytes), adding an ellipsis.
fn truncate(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    let mut out: String = text.chars().take(max.saturating_sub(1)).collect();
    out.push('…');
    out
}

/// A thread title from the opening words of the first message, cut on a word
/// boundary when it has to be shortened.
pub fn title_from(message: &str) -> String {
    let trimmed = message.trim();
    let short: String = trimmed.chars().take(MAX_TITLE_CHARS).collect();
    if short.chars().count() == trimmed.chars().count() {
        return short;
    }
    match short.rfind(' ') {
        Some(space) if space >= 12 => format!("{}…", short[..space].trim_end()),
        _ => format!("{short}…"),
    }
}

/// Persist one chat turn: the user's message and the assistant's reply, then
/// bump the thread (giving it a title on first use).
/// Returns the refreshed conversation and the two saved lines, in order.
pub async fn persist(
    conn: &mut SqliteConnection,
    conversation_id: Option<i64>,
    user_text: &str,
    assistant_text: &str,
) -> Result<(AiConversation, Vec<AiMessage>), AppError> {
    let conversation = match conversation_id {
        Some(id) => repo::find_conversation(conn, id)
            .await?
            .ok_or(AppError::NotFound)?,
        None => repo::insert_conversation(conn, None).await?,
    };

    let user = repo::insert_message(conn, conversation.id, "user", user_text).await?;
    let assistant =
        repo::insert_message(conn, conversation.id, "assistant", assistant_text).await?;

    let title = conversation
        .title
        .is_none()
        .then(|| title_from(user_text));
    let conversation = repo::touch_conversation(conn, conversation.id, title.as_deref()).await?;

    Ok((conversation, vec![user, assistant]))
}

/// Convenience for building the model's reply turn from response blocks.
pub fn assistant_blocks(text: &str) -> Vec<ContentBlock> {
    vec![ContentBlock::text(text)]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stored(role: &str, content: &str) -> AiMessage {
        AiMessage {
            id: 1,
            conversation_id: 1,
            role: role.to_string(),
            content: content.to_string(),
            created_at: "2026-10-16T08:00:00Z".to_string(),
        }
    }

    #[test]
    fn titles_keep_short_messages_whole() {
        assert_eq!(title_from("Plan my day"), "Plan my day");
    }

    #[test]
    fn titles_cut_long_messages_on_a_word_boundary() {
        let message = "Please create a task for every single thing on my shopping list today";
        let title = title_from(message);
        assert!(title.chars().count() <= 49, "title too long: {title}");
        assert!(title.ends_with('…'));
        assert!(!title.contains(" today"));
    }

    #[test]
    fn titles_handle_unbroken_text_without_panicking_on_utf8() {
        let title = title_from(&"ä".repeat(100));
        assert_eq!(title.chars().count(), MAX_TITLE_CHARS + 1); // 48 + ellipsis
    }

    #[test]
    fn history_maps_to_alternating_text_messages() {
        let history = vec![
            stored("user", "Hello"),
            stored("assistant", "Hi there"),
        ];
        let messages = to_provider_messages(&history);
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, "user");
        assert_eq!(messages[1].role, "assistant");
        assert!(matches!(&messages[0].content[0], ContentBlock::Text(t) if t == "Hello"));
    }
}
