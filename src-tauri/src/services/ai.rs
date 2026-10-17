//! Phase 5 business rules: settings, threads and the chat loop
//! (context -> model -> validated tool calls -> persistence).

use sqlx::SqlitePool;

use crate::ai::openai::OpenAiProvider;
use crate::ai::provider::{
    AiProvider, AnthropicProvider, ContentBlock, ProviderMessage, ProviderRequest,
};
use crate::ai::validation::ToolCall;
use crate::ai::{context, conversation, secrets, tools, validation};
use crate::errors::AppError;
use crate::models::ai::{
    AiConversation, AiMessage, AiSettingsInput, AiSettingsView, ChatInput, ChatResult,
};
use crate::models::gamification::XpOutcome;
use crate::repositories::ai as repo;
use crate::services::{gamification, quest};

/// Longest message the user may send in one turn. Kept modest so a single turn
/// stays inside a free tier's per-minute token budget.
const MAX_MESSAGE_CHARS: usize = 2000;
/// History messages (per side) fed back to the model. Small on purpose: the
/// live context below already carries the app state the model needs.
const MAX_HISTORY: i64 = 6;
/// Each history line is truncated to this many characters before sending.
const HISTORY_MESSAGE_CHARS: usize = 500;
/// History returned when the UI opens a thread.
const MAX_THREAD_MESSAGES: i64 = 200;
/// Threads listed in the sidebar.
const MAX_CONVERSATIONS: i64 = 50;
/// Safety valve for model / tool ping-pong inside one turn.
const MAX_TOOL_ROUNDS: usize = 4;
/// Cap on the model's reply length per round.
const CHAT_MAX_TOKENS: u32 = 1024;
/// Newest model id by default; the user can change it in the panel.
const DEFAULT_MODEL: &str = "claude-sonnet-5-5";

const KEY_API_KEY: &str = "anthropic_api_key";
const KEY_MODEL: &str = "ai_model";
const KEY_BASE_URL: &str = "ai_base_url";

const SYSTEM_PROMPT: &str = "You are JARVIS, the assistant built into Habit Planner, a desktop habit and task app with XP, levels, quests and goals.

You help the user plan their day and review progress.
- Answer briefly and concretely; the live data below is the source of truth for ids, counts and dates.
- When the user asks you to create or complete something, call the matching tool instead of only describing it, then report in one sentence exactly what you did.
- Never invent ids: use the ids from the live data. If you do not have an id, list what you can see instead.
- If a tool call fails, read its error, correct the call or explain what went wrong; never repeat the same failing call unchanged.
- Dates are local to the user and look like 2026-10-16 (or 2026-10-16T14:30 for a time).
- Keep replies to a few sentences unless the user asks for detail.";

struct Settings {
    api_key: Option<String>,
    /// None => Anthropic; Some => an OpenAI-compatible endpoint.
    base_url: Option<String>,
    model: String,
}

pub struct AiService {
    pool: SqlitePool,
    /// Present only when running as a real app; used for secure credential
    /// storage. Tests build the service without one and use the DB fallback.
    app: Option<tauri::AppHandle>,
}

impl AiService {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool, app: None }
    }

    /// Attach the app handle so the credential is kept in secure storage.
    pub fn with_app(mut self, app: tauri::AppHandle) -> Self {
        self.app = Some(app);
        self
    }

    // ---- credential storage ----------------------------------------------

    /// The stored credential, reading from secure storage and migrating any
    /// legacy plaintext key found in the database.
    async fn load_api_key(&self) -> Result<Option<String>, AppError> {
        if let Some(app) = &self.app {
            if let Some(key) = secrets::load(app)? {
                return Ok(Some(key));
            }
        }

        let mut conn = self.pool.acquire().await?;
        let legacy = repo::get_setting(&mut conn, KEY_API_KEY)
            .await?
            .filter(|key| !key.trim().is_empty());
        drop(conn);

        // Migrate: move a legacy plaintext key into secure storage and drop it
        // from the database so it is never written to disk in the clear again.
        if let (Some(app), Some(key)) = (&self.app, &legacy) {
            secrets::store(app, key)?;
            let mut conn = self.pool.acquire().await?;
            repo::delete_setting(&mut conn, KEY_API_KEY).await?;
        }

        Ok(legacy)
    }

    async fn store_api_key(&self, key: &str) -> Result<(), AppError> {
        match &self.app {
            Some(app) => {
                secrets::store(app, key)?;
                // Drop any legacy plaintext copy.
                let mut conn = self.pool.acquire().await?;
                repo::delete_setting(&mut conn, KEY_API_KEY).await
            }
            // No secure storage available (tests): keep the legacy behaviour.
            None => {
                let mut conn = self.pool.acquire().await?;
                repo::set_setting(&mut conn, KEY_API_KEY, key).await
            }
        }
    }

    async fn clear_api_key(&self) -> Result<(), AppError> {
        if let Some(app) = &self.app {
            secrets::clear(app)?;
        }
        let mut conn = self.pool.acquire().await?;
        repo::delete_setting(&mut conn, KEY_API_KEY).await
    }

    // ---- settings ---------------------------------------------------------

    async fn settings(&self) -> Result<Settings, AppError> {
        let mut conn = self.pool.acquire().await?;
        let base_url = repo::get_setting(&mut conn, KEY_BASE_URL).await?;
        let model = repo::get_setting(&mut conn, KEY_MODEL)
            .await?
            .filter(|m| !m.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_MODEL.to_string());
        drop(conn);

        let api_key = self.load_api_key().await?;

        Ok(Settings {
            api_key,
            base_url: base_url.filter(|u| !u.trim().is_empty()),
            model,
        })
    }

    /// What the UI may see. The stored key is only ever reported as "set".
    pub async fn get_settings(&self) -> Result<AiSettingsView, AppError> {
        let settings = self.settings().await?;
        Ok(AiSettingsView {
            api_key_set: settings.api_key.is_some(),
            base_url: settings.base_url.unwrap_or_default(),
            model: settings.model,
        })
    }

    pub async fn set_settings(&self, input: AiSettingsInput) -> Result<AiSettingsView, AppError> {
        if let Some(key) = &input.api_key {
            let key = key.trim();
            if key.is_empty() {
                self.clear_api_key().await?;
            } else if key.len() > 512 || key.chars().any(char::is_whitespace) {
                return Err(AppError::InvalidInput(
                    "That API key looks wrong: expected at most 512 characters with no spaces."
                        .into(),
                ));
            } else {
                self.store_api_key(key).await?;
            }
        }

        let mut conn = self.pool.acquire().await?;

        if let Some(base_url) = &input.base_url {
            let base_url = base_url.trim();
            if base_url.is_empty() {
                // Clearing the field goes back to the built-in Anthropic provider.
                repo::delete_setting(&mut conn, KEY_BASE_URL).await?;
            } else if base_url.len() > 300
                || base_url.chars().any(char::is_whitespace)
                || !(base_url.starts_with("http://") || base_url.starts_with("https://"))
            {
                return Err(AppError::InvalidInput(
                    "The base URL must start with http:// or https:// and contain no spaces.".into(),
                ));
            } else {
                repo::set_setting(&mut conn, KEY_BASE_URL, base_url).await?;
            }
        }

        if let Some(model) = &input.model {
            let model = model.trim();
            if model.is_empty() {
                // Clearing the field goes back to the built-in default.
                repo::delete_setting(&mut conn, KEY_MODEL).await?;
            } else if model.len() > 100
                || !model.chars().all(|c| {
                    c.is_ascii_alphanumeric() || matches!(c, '-' | '.' | '_' | ':' | '/')
                })
            {
                // ':' and '/' are allowed for local tags (llama3.1:8b) and hosted
                // names (vendor/model:free).
                return Err(AppError::InvalidInput(
                    "A model id may only contain letters, digits, '-', '.', '_', ':' and '/' (max 100)."
                        .into(),
                ));
            } else {
                repo::set_setting(&mut conn, KEY_MODEL, model).await?;
            }
        }

        drop(conn);
        self.get_settings().await
    }

    // ---- threads ----------------------------------------------------------

    pub async fn list_conversations(&self) -> Result<Vec<AiConversation>, AppError> {
        let mut conn = self.pool.acquire().await?;
        repo::list_conversations(&mut conn, MAX_CONVERSATIONS).await
    }

    pub async fn get_conversation(&self, id: i64) -> Result<Vec<AiMessage>, AppError> {
        let mut conn = self.pool.acquire().await?;
        repo::find_conversation(&mut conn, id)
            .await?
            .ok_or(AppError::NotFound)?;
        repo::list_messages(&mut conn, id, MAX_THREAD_MESSAGES).await
    }

    pub async fn delete_conversation(&self, id: i64) -> Result<(), AppError> {
        let mut conn = self.pool.acquire().await?;
        if repo::delete_conversation(&mut conn, id).await? {
            Ok(())
        } else {
            Err(AppError::NotFound)
        }
    }

    // ---- chat -------------------------------------------------------------

    /// Build the provider from stored settings. A base URL selects the
    /// OpenAI-compatible path (Groq, Ollama, a self-hosted proxy, ...);
    /// with no base URL the built-in Anthropic provider is used.
    fn provider(&self, settings: &Settings) -> Result<Box<dyn AiProvider>, AppError> {
        match settings.base_url.clone() {
            Some(base_url) => Ok(Box::new(OpenAiProvider::new(
                settings.api_key.clone(),
                base_url,
                settings.model.clone(),
            )?)),
            None => {
                let api_key = settings.api_key.clone().ok_or_else(|| {
                    AppError::InvalidInput(
                        "Add an API key (gear icon above the chat), or set a base URL to use a free OpenAI-compatible service.".into(),
                    )
                })?;
                Ok(Box::new(AnthropicProvider::new(api_key, settings.model.clone())?))
            }
        }
    }

    /// One non-streaming chat turn with the configured provider.
    pub async fn chat(&self, input: ChatInput) -> Result<ChatResult, AppError> {
        let settings = self.settings().await?;
        let provider = self.provider(&settings)?;
        self.run(&input, provider.as_ref(), |_delta: &str| {}).await
    }

    /// One chat turn that reports assistant text as it streams in. `on_text` is
    /// called for every chunk (useful for showing words on a slow connection).
    pub async fn chat_streaming<F>(
        &self,
        input: ChatInput,
        on_text: F,
    ) -> Result<ChatResult, AppError>
    where
        F: FnMut(&str) + Send,
    {
        let settings = self.settings().await?;
        let provider = self.provider(&settings)?;
        self.run(&input, provider.as_ref(), on_text).await
    }

    /// The chat loop with an injected provider, so tests can drive it without a network.
    pub async fn chat_with(
        &self,
        input: &ChatInput,
        provider: &dyn AiProvider,
    ) -> Result<ChatResult, AppError> {
        self.run(input, provider, |_delta: &str| {}).await
    }

    /// The chat loop: context -> model -> validated tool calls -> persistence.
    /// Text the model emits is passed to `on_text` as it arrives.
    async fn run<F>(
        &self,
        input: &ChatInput,
        provider: &dyn AiProvider,
        mut on_text: F,
    ) -> Result<ChatResult, AppError>
    where
        F: FnMut(&str) + Send,
    {
        let user_text = input.message.trim();
        if user_text.is_empty() {
            return Err(AppError::InvalidInput("Message cannot be empty".into()));
        }
        if user_text.chars().count() > MAX_MESSAGE_CHARS {
            return Err(AppError::InvalidInput(format!(
                "Messages are limited to {MAX_MESSAGE_CHARS} characters"
            )));
        }

        let today = quest::local_today().to_string();

        // Existing thread: load its history. A new one starts empty and is
        // only created once there is a reply worth keeping.
        let (conversation_id, history) = match input.conversation_id {
            Some(id) => {
                let mut conn = self.pool.acquire().await?;
                repo::find_conversation(&mut conn, id)
                    .await?
                    .ok_or(AppError::NotFound)?;
                let history = repo::list_messages(&mut conn, id, MAX_HISTORY).await?;
                drop(conn);
                (Some(id), history)
            }
            None => (None, Vec::new()),
        };

        let system = format!(
            "{SYSTEM_PROMPT}\n\n--- LIVE DATA (source of truth) ---\n{}",
            context::build(&self.pool, &today).await?
        );

        // Old turns are truncated before they are sent: a single long message
        // must not consume the whole per-minute budget.
        let mut messages = conversation::to_provider_messages_capped(&history, HISTORY_MESSAGE_CHARS);
        messages.push(ProviderMessage::user(vec![ContentBlock::text(user_text)]));

        let tool_defs = tools::definitions();
        let mut xp_delta = 0i64;
        let mut leveled_up = false;
        let mut profile = None;
        let mut said: Vec<String> = Vec::new();
        let mut final_text: Option<String> = None;

        for _ in 0..MAX_TOOL_ROUNDS {
            let response = provider
                .complete_streaming(
                    ProviderRequest {
                        system: system.clone(),
                        messages: messages.clone(),
                        tools: tool_defs.clone(),
                        max_tokens: CHAT_MAX_TOKENS,
                    },
                    Box::new(|delta: String| on_text(&delta)),
                )
                .await?;

            let tool_uses = response.tool_uses();
            if tool_uses.is_empty() {
                final_text = Some(response.text());
                break;
            }

            // Keep the assistant turn verbatim (thinking blocks included) so the
            // API accepts it when we send the tool results back.
            let text = response.text();
            if !text.trim().is_empty() {
                said.push(text);
            }
            messages.push(ProviderMessage::assistant(response.content));

            let mut results = Vec::with_capacity(tool_uses.len());
            for (id, name, input) in tool_uses {
                let call = ToolCall {
                    id: id.clone(),
                    name,
                    input,
                };
                let outcome = match validation::validate(&call) {
                    Err(error) => tools::ToolOutcome {
                        result: error,
                        is_error: true,
                        xp: None,
                    },
                    Ok(validated) => tools::execute(&self.pool, validated, &today).await?,
                };
                if let Some(xp) = &outcome.xp {
                    xp_delta += xp.xp_delta;
                    leveled_up |= xp.leveled_up;
                    profile = Some(xp.profile.clone());
                }
                results.push(ContentBlock::ToolResult {
                    tool_use_id: id,
                    content: outcome.result,
                    is_error: outcome.is_error,
                });
            }
            messages.push(ProviderMessage::user(results));
        }

        let reply = match final_text {
            Some(text) if !text.trim().is_empty() => text,
            _ if !said.is_empty() => said.join("\n"),
            _ => "Done — I've made those changes.".to_string(),
        };

        // Persist the turn (user line + assistant line) and any XP the tools
        // earned, in one transaction.
        let mut tx = self.pool.begin().await?;
        let (thread, saved) =
            conversation::persist(&mut tx, conversation_id, user_text, &reply).await?;
        let xp = match profile {
            Some(profile) => XpOutcome {
                xp_delta,
                leveled_up,
                profile,
            },
            None => gamification::current_outcome(&mut tx).await?,
        };
        tx.commit().await?;

        Ok(ChatResult {
            conversation: thread,
            messages: saved,
            xp,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(message: &str) -> ChatInput {
        ChatInput {
            conversation_id: None,
            message: message.to_string(),
        }
    }

    #[test]
    fn system_prompt_names_the_tools_and_the_live_data() {
        assert!(SYSTEM_PROMPT.contains("live data"));
        assert!(SYSTEM_PROMPT.contains("tool"));
        assert!(!SYSTEM_PROMPT.contains("sk-ant"), "no key material in the prompt");
    }

    #[test]
    fn defaults_and_message_shapes_are_usable() {
        assert!(
            DEFAULT_MODEL.starts_with("claude-"),
            "unexpected default model: {DEFAULT_MODEL}"
        );
        let message = input("hello");
        assert_eq!(message.conversation_id, None, "fresh thread by default");
        assert_eq!(message.message, "hello");
    }
}
