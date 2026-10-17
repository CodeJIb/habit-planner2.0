//! Provider abstraction: everything that talks to a model goes through
//! [`AiProvider`], so API keys never reach the frontend and swapping the
//! model vendor only means swapping the implementation.

use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use serde::Serialize;
use serde_json::{json, Value};

use crate::errors::AppError;

/// Boxed future so the trait stays object-safe without extra dependencies.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Called with each chunk of assistant text as it streams in. Used to push
/// partial text to the UI so slow mobile connections show words immediately.
pub type TextSink<'a> = Box<dyn FnMut(String) + Send + 'a>;

pub const ANTHROPIC_ENDPOINT: &str = "https://api.anthropic.com/v1/messages";
pub const ANTHROPIC_VERSION: &str = "2023-06-01";
pub const DEFAULT_MAX_TOKENS: u32 = 8192;

/// One block of a message's content. `Raw` carries blocks we do not model
/// (e.g. thinking blocks) straight back to the API untouched.
#[derive(Debug, Clone)]
pub enum ContentBlock {
    Text(String),
    ToolUse {
        id: String,
        name: String,
        input: Value,
    },
    ToolResult {
        tool_use_id: String,
        content: String,
        is_error: bool,
    },
    Raw(Value),
}

impl ContentBlock {
    pub fn text(text: impl Into<String>) -> Self {
        ContentBlock::Text(text.into())
    }
}

// The Anthropic wire format: each block is an object tagged with "type".
// `Raw` blocks are already wire-shaped, so they are emitted as-is.
impl Serialize for ContentBlock {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let value = match self {
            ContentBlock::Text(text) => json!({ "type": "text", "text": text }),
            ContentBlock::ToolUse { id, name, input } => {
                json!({ "type": "tool_use", "id": id, "name": name, "input": input })
            }
            ContentBlock::ToolResult {
                tool_use_id,
                content,
                is_error,
            } => json!({
                "type": "tool_result",
                "tool_use_id": tool_use_id,
                "content": content,
                "is_error": is_error,
            }),
            ContentBlock::Raw(value) => value.clone(),
        };
        value.serialize(serializer)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ProviderMessage {
    pub role: String,
    pub content: Vec<ContentBlock>,
}

impl ProviderMessage {
    pub fn user(content: Vec<ContentBlock>) -> Self {
        Self {
            role: "user".to_string(),
            content,
        }
    }

    pub fn assistant(content: Vec<ContentBlock>) -> Self {
        Self {
            role: "assistant".to_string(),
            content,
        }
    }

    pub fn text(role: &str, text: &str) -> Self {
        Self {
            role: role.to_string(),
            content: vec![ContentBlock::text(text)],
        }
    }
}

/// A complete chat request in provider terms (already includes the app context).
#[derive(Debug, Clone)]
pub struct ProviderRequest {
    pub system: String,
    pub messages: Vec<ProviderMessage>,
    pub tools: Vec<Value>,
    pub max_tokens: u32,
}

#[derive(Debug, Clone)]
pub struct ProviderResponse {
    pub content: Vec<ContentBlock>,
    pub stop_reason: Option<String>,
}

impl ProviderResponse {
    /// The assistant's visible words (thinking blocks are not text).
    pub fn text(&self) -> String {
        self.content
            .iter()
            .filter_map(|block| match block {
                ContentBlock::Text(text) => Some(text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn tool_uses(&self) -> Vec<(String, String, Value)> {
        self.content
            .iter()
            .filter_map(|block| match block {
                ContentBlock::ToolUse { id, name, input } => {
                    Some((id.clone(), name.clone(), input.clone()))
                }
                _ => None,
            })
            .collect()
    }
}

/// A chat model backend. Implementations must be cheap to share and may not
/// leak credentials to the frontend.
pub trait AiProvider: Send + Sync {
    fn complete<'a>(
        &'a self,
        request: ProviderRequest,
    ) -> BoxFuture<'a, Result<ProviderResponse, AppError>>;

    /// Like [`AiProvider::complete`], but reports assistant text as it arrives
    /// through `sink`. Providers without streaming may ignore the sink.
    fn complete_streaming<'a>(
        &'a self,
        request: ProviderRequest,
        sink: TextSink<'a>,
    ) -> BoxFuture<'a, Result<ProviderResponse, AppError>>;
}

pub struct AnthropicProvider {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

impl AnthropicProvider {
    pub fn new(api_key: String, model: String) -> Result<Self, AppError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(120))
            .build()?;
        Ok(Self {
            api_key,
            model,
            client,
        })
    }

    fn body(&self, request: &ProviderRequest) -> Value {
        let mut body = serde_json::Map::new();
        body.insert("model".into(), json!(self.model));
        body.insert("max_tokens".into(), json!(request.max_tokens));
        body.insert("system".into(), json!(request.system));
        body.insert(
            "messages".into(),
            serde_json::to_value(&request.messages).unwrap_or_else(|_| json!([])),
        );
        if !request.tools.is_empty() {
            body.insert("tools".into(), json!(request.tools));
        }
        Value::Object(body)
    }
}

impl AiProvider for AnthropicProvider {
    fn complete<'a>(
        &'a self,
        request: ProviderRequest,
    ) -> BoxFuture<'a, Result<ProviderResponse, AppError>> {
        Box::pin(async move {
            let response = self
                .client
                .post(ANTHROPIC_ENDPOINT)
                .header("x-api-key", &self.api_key)
                .header("anthropic-version", ANTHROPIC_VERSION)
                .header("content-type", "application/json")
                .json(&self.body(&request))
                .send()
                .await?;

            let status = response.status();
            let bytes = response.bytes().await?;
            let value: Value = serde_json::from_slice(&bytes).map_err(|e| {
                AppError::Ai(format!("Anthropic sent a response that is not JSON: {e}"))
            })?;

            if !status.is_success() {
                let message = value
                    .pointer("/error/message")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .unwrap_or_else(|| value.to_string());
                return Err(AppError::Ai(format!(
                    "Anthropic API error ({}): {message}",
                    status.as_u16()
                )));
            }
            parse_response(value)
        })
    }

    /// Anthropic's Messages API supports SSE streaming, but this app only
    /// streams the OpenAI-compatible path; here we simply return the full reply.
    fn complete_streaming<'a>(
        &'a self,
        request: ProviderRequest,
        _sink: TextSink<'a>,
    ) -> BoxFuture<'a, Result<ProviderResponse, AppError>> {
        self.complete(request)
    }
}

/// Turn a Messages API response body into a [`ProviderResponse`].
/// Blocks we do not understand (thinking, citations, ...) survive as `Raw`.
fn parse_response(value: Value) -> Result<ProviderResponse, AppError> {
    let blocks = value
        .get("content")
        .and_then(Value::as_array)
        .ok_or_else(|| AppError::Ai("Anthropic response has no content array".into()))?;

    let content = blocks
        .iter()
        .map(|block| match block.get("type").and_then(Value::as_str) {
            Some("text") => ContentBlock::Text(
                block
                    .get("text")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
            ),
            Some("tool_use") => ContentBlock::ToolUse {
                id: block
                    .get("id")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                name: block
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                input: block.get("input").cloned().unwrap_or_else(|| json!({})),
            },
            _ => ContentBlock::Raw(block.clone()),
        })
        .collect();

    Ok(ProviderResponse {
        content,
        stop_reason: value
            .get("stop_reason")
            .and_then(Value::as_str)
            .map(str::to_string),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_messages_in_anthropic_shape() {
        let request = ProviderRequest {
            system: "You are JARVIS.".into(),
            messages: vec![
                ProviderMessage::text("user", "Plan my day"),
                ProviderMessage::assistant(vec![
                    ContentBlock::text("On it."),
                    ContentBlock::ToolUse {
                        id: "toolu_1".into(),
                        name: "create_task".into(),
                        input: json!({ "title": "Write report" }),
                    },
                ]),
                ProviderMessage::user(vec![ContentBlock::ToolResult {
                    tool_use_id: "toolu_1".into(),
                    content: "Created task #1.".into(),
                    is_error: false,
                }]),
            ],
            tools: vec![json!({"name": "create_task"})],
            max_tokens: 1024,
        };

        let value = serde_json::to_value(&request.messages).unwrap();
        assert_eq!(value[0]["role"], "user");
        assert_eq!(value[0]["content"][0]["type"], "text");
        assert_eq!(value[0]["content"][0]["text"], "Plan my day");
        assert_eq!(value[1]["content"][1]["type"], "tool_use");
        assert_eq!(value[1]["content"][1]["input"]["title"], "Write report");
        assert_eq!(value[2]["content"][0]["type"], "tool_result");
        assert_eq!(value[2]["content"][0]["tool_use_id"], "toolu_1");
    }

    #[test]
    fn raw_blocks_pass_through_untouched() {
        let thinking = json!({ "type": "thinking", "thinking": "hmm", "signature": "abc" });
        let block = ContentBlock::Raw(thinking.clone());
        assert_eq!(serde_json::to_value(block).unwrap(), thinking);
    }

    #[test]
    fn parses_text_tool_use_and_unknown_blocks() {
        let body = json!({
            "id": "msg_1",
            "type": "message",
            "role": "assistant",
            "content": [
                { "type": "thinking", "thinking": "planning", "signature": "sig" },
                { "type": "text", "text": "Creating that now." },
                { "type": "tool_use", "id": "toolu_9", "name": "create_habit",
                  "input": { "title": "Meditate" } }
            ],
            "stop_reason": "tool_use",
            "usage": { "input_tokens": 10, "output_tokens": 20 }
        });

        let response = parse_response(body).unwrap();
        assert_eq!(response.stop_reason.as_deref(), Some("tool_use"));
        assert_eq!(response.text(), "Creating that now.");

        let tools = response.tool_uses();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].0, "toolu_9");
        assert_eq!(tools[0].1, "create_habit");
        assert_eq!(tools[0].2["title"], "Meditate");

        // The thinking block is kept verbatim so it can be sent back.
        assert!(matches!(&response.content[0], ContentBlock::Raw(v) if v["signature"] == "sig"));
    }

    #[test]
    fn missing_content_is_an_error() {
        assert!(parse_response(json!({ "id": "msg_1" })).is_err());
    }

    #[test]
    fn body_omits_empty_tools_and_carries_the_system_prompt() {
        let provider = AnthropicProvider::new("key".into(), "claude-test".into()).unwrap();
        let request = ProviderRequest {
            system: "context here".into(),
            messages: vec![ProviderMessage::text("user", "hi")],
            tools: vec![],
            max_tokens: 512,
        };
        let body = provider.body(&request);
        assert_eq!(body["model"], "claude-test");
        assert_eq!(body["system"], "context here");
        assert_eq!(body["max_tokens"], 512);
        assert!(body.get("tools").is_none());

        let with_tools = ProviderRequest {
            tools: vec![json!({"name": "create_task"})],
            ..request
        };
        assert!(provider.body(&with_tools).get("tools").is_some());
    }
}
