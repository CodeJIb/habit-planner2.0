//! OpenAI-compatible provider: one HTTP client that speaks the
//! `/chat/completions` wire format, so the assistant can run against any
//! compatible backend — a local Ollama (`http://localhost:11434/v1`), Google
//! Gemini's OpenAI endpoint, Groq, OpenRouter, LM Studio, a self-hosted proxy,
//! and so on.
//!
//! The internal [`ProviderMessage`]/[`ContentBlock`] shape is Anthropic-flavoured,
//! so this module translates to and from the OpenAI shape on every call.
//!
//! Everything here is tuned for low-end phones on slow, flaky mobile data:
//! requests fail fast when the link is dead, retry with backoff on `429`/`5xx`,
//! and can stream so the first words appear while the rest is still arriving.

use std::time::Duration;

use futures_util::StreamExt;
use serde_json::{json, Value};
use tokio::time::sleep;

use super::provider::{
    AiProvider, BoxFuture, ContentBlock, ProviderMessage, ProviderRequest, ProviderResponse,
    TextSink,
};
use crate::errors::AppError;

/// Total attempts (first try + retries) for 429 / 5xx / transient links.
const MAX_ATTEMPTS: u32 = 3;
/// First backoff step; doubles each attempt and is capped at `MAX_BACKOFF_MS`.
const BASE_BACKOFF_MS: u64 = 500;
const MAX_BACKOFF_MS: u64 = 8_000;
/// Even if the server asks for a longer `Retry-After`, never sleep past this.
const MAX_RETRY_AFTER_MS: u64 = 12_000;
/// Fail fast on a dead mobile link instead of hanging.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
/// Whole-request ceiling: long enough for a slow model, short enough that a
/// stalled request does not tie up the UI forever.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(90);

pub struct OpenAiProvider {
    /// Ollama and other local servers (and a key-injecting proxy) need no key,
    /// so this is optional.
    api_key: Option<String>,
    /// Base URL without a trailing slash, e.g. `http://localhost:11434/v1`.
    base_url: String,
    model: String,
    client: reqwest::Client,
}

impl OpenAiProvider {
    pub fn new(
        api_key: Option<String>,
        base_url: String,
        model: String,
    ) -> Result<Self, AppError> {
        let client = reqwest::Client::builder()
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(REQUEST_TIMEOUT)
            .build()?;
        Ok(Self {
            api_key: api_key.filter(|key| !key.trim().is_empty()),
            base_url: base_url.trim().trim_end_matches('/').to_string(),
            model,
            client,
        })
    }

    fn endpoint(&self) -> String {
        format!("{}/chat/completions", self.base_url)
    }

    fn body(&self, request: &ProviderRequest, stream: bool) -> Value {
        let mut messages = vec![json!({ "role": "system", "content": request.system })];
        for message in &request.messages {
            messages.extend(translate_message(message));
        }

        let mut body = serde_json::Map::new();
        body.insert("model".into(), json!(self.model));
        body.insert("max_tokens".into(), json!(request.max_tokens));
        body.insert("messages".into(), json!(messages));
        if !request.tools.is_empty() {
            body.insert("tools".into(), json!(openai_tools(&request.tools)));
        }
        if stream {
            body.insert("stream".into(), json!(true));
        }
        Value::Object(body)
    }

    fn request(&self, body: &Value) -> reqwest::RequestBuilder {
        let mut builder = self
            .client
            .post(self.endpoint())
            .header("content-type", "application/json");
        if let Some(key) = &self.api_key {
            builder = builder.header("authorization", format!("Bearer {key}"));
        }
        builder.json(body)
    }

    /// POST the body, retrying `429` / `5xx` and transient connection failures
    /// with exponential backoff (honouring `Retry-After` when present).
    async fn send_with_retry(&self, body: &Value) -> Result<reqwest::Response, AppError> {
        let mut attempt = 0;
        loop {
            attempt += 1;
            match self.request(body).send().await {
                Ok(response) => {
                    let status = response.status();
                    if is_retryable_status(status) && attempt < MAX_ATTEMPTS {
                        sleep(backoff_delay(attempt, parse_retry_after(response.headers()))).await;
                        continue;
                    }
                    return Ok(response);
                }
                Err(error) if attempt < MAX_ATTEMPTS && is_transient(&error) => {
                    sleep(backoff_delay(attempt, None)).await;
                    continue;
                }
                Err(error) => return Err(map_transport_error(error)),
            }
        }
    }
}

impl AiProvider for OpenAiProvider {
    fn complete<'a>(
        &'a self,
        request: ProviderRequest,
    ) -> BoxFuture<'a, Result<ProviderResponse, AppError>> {
        Box::pin(async move {
            let body = self.body(&request, false);
            let response = self.send_with_retry(&body).await?;
            let status = response.status();
            let bytes = response.bytes().await.map_err(map_transport_error)?;
            interpret(status, &bytes)
        })
    }

    fn complete_streaming<'a>(
        &'a self,
        request: ProviderRequest,
        mut sink: TextSink<'a>,
    ) -> BoxFuture<'a, Result<ProviderResponse, AppError>> {
        Box::pin(async move {
            let body = self.body(&request, true);
            let response = self.send_with_retry(&body).await?;
            let status = response.status();

            // Errors are never streamed: read the whole body and report it.
            if !status.is_success() {
                let bytes = response.bytes().await.map_err(map_transport_error)?;
                return interpret(status, &bytes);
            }

            let mut stream = response.bytes_stream();
            let mut assembler = StreamAssembler::default();
            while let Some(chunk) = stream.next().await {
                let chunk = chunk.map_err(map_transport_error)?;
                for delta in assembler.push(&String::from_utf8_lossy(&chunk)) {
                    sink(delta);
                }
            }
            assembler.finish()
        })
    }
}

/// Translate one internal message into the OpenAI messages it becomes. A single
/// internal message can fan out (a user turn holding several tool results becomes
/// one `tool` message each).
fn translate_message(message: &ProviderMessage) -> Vec<Value> {
    let mut texts: Vec<String> = Vec::new();
    let mut tool_calls: Vec<Value> = Vec::new();
    let mut tool_results: Vec<Value> = Vec::new();

    for block in &message.content {
        match block {
            ContentBlock::Text(text) => texts.push(text.clone()),
            ContentBlock::ToolUse { id, name, input } => tool_calls.push(json!({
                "id": id,
                "type": "function",
                "function": {
                    "name": name,
                    // OpenAI carries arguments as a JSON-encoded string.
                    "arguments": serde_json::to_string(input).unwrap_or_else(|_| "{}".into()),
                }
            })),
            ContentBlock::ToolResult {
                tool_use_id,
                content,
                is_error,
            } => {
                // OpenAI has no error flag, so failure is spelled out in the text.
                let content = if *is_error {
                    format!("Error: {content}")
                } else {
                    content.clone()
                };
                tool_results.push(json!({
                    "role": "tool",
                    "tool_call_id": tool_use_id,
                    "content": content,
                }));
            }
            // Thinking blocks have no OpenAI equivalent; they are dropped.
            ContentBlock::Raw(_) => {}
        }
    }

    let joined = texts.join("\n");
    if !tool_results.is_empty() {
        let mut out = tool_results;
        if !joined.is_empty() {
            out.push(json!({ "role": "user", "content": joined }));
        }
        out
    } else if !tool_calls.is_empty() {
        vec![json!({
            "role": "assistant",
            "content": if joined.is_empty() { Value::Null } else { json!(joined) },
            "tool_calls": tool_calls,
        })]
    } else {
        vec![json!({ "role": message.role, "content": joined })]
    }
}

/// Wrap the app's tool schemas (`name`/`description`/`input_schema`) in the
/// OpenAI `tools` envelope (`type: function`, `function.parameters`).
fn openai_tools(definitions: &[Value]) -> Vec<Value> {
    definitions
        .iter()
        .map(|def| {
            json!({
                "type": "function",
                "function": {
                    "name": def.get("name").cloned().unwrap_or(Value::Null),
                    "description": def.get("description").cloned().unwrap_or(Value::Null),
                    "parameters": def
                        .get("input_schema")
                        .cloned()
                        .unwrap_or_else(|| json!({ "type": "object", "properties": {} })),
                }
            })
        })
        .collect()
}

/// Turn a `/chat/completions` response body into a [`ProviderResponse`].
fn parse_response(value: Value) -> Result<ProviderResponse, AppError> {
    let choice = value
        .pointer("/choices/0")
        .ok_or_else(|| AppError::Ai("The model response has no choices".into()))?;
    let message = choice
        .get("message")
        .ok_or_else(|| AppError::Ai("The model response has no message".into()))?;

    let mut content = Vec::new();
    if let Some(text) = message.get("content").and_then(Value::as_str) {
        if !text.is_empty() {
            content.push(ContentBlock::Text(text.to_string()));
        }
    }

    if let Some(calls) = message.get("tool_calls").and_then(Value::as_array) {
        for call in calls {
            let function = call.get("function");
            content.push(ContentBlock::ToolUse {
                id: call
                    .get("id")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                name: function
                    .and_then(|f| f.get("name"))
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                input: function
                    .and_then(|f| f.get("arguments"))
                    .and_then(Value::as_str)
                    .and_then(|args| serde_json::from_str(args).ok())
                    .unwrap_or_else(|| json!({})),
            });
        }
    }

    Ok(ProviderResponse {
        content,
        stop_reason: choice
            .get("finish_reason")
            .and_then(Value::as_str)
            .map(str::to_string),
    })
}

/// Parse a successful/errorful response body into a result, turning HTTP status
/// codes into plain-language errors the UI can show as-is.
fn interpret(status: reqwest::StatusCode, bytes: &[u8]) -> Result<ProviderResponse, AppError> {
    let value: Value = match serde_json::from_slice(bytes) {
        Ok(value) => value,
        Err(_) if !status.is_success() => {
            return Err(status_error(status, &String::from_utf8_lossy(bytes)));
        }
        Err(e) => {
            return Err(AppError::Ai(format!(
                "The model returned a response that is not JSON: {e}"
            )));
        }
    };

    if !status.is_success() {
        let message = value
            .pointer("/error/message")
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap_or_else(|| value.to_string());
        return Err(status_error(status, &message));
    }

    parse_response(value)
}

/// Only these are worth retrying: the request may succeed a moment later.
fn is_retryable_status(status: reqwest::StatusCode) -> bool {
    status == reqwest::StatusCode::TOO_MANY_REQUESTS || status.is_server_error()
}

/// Connection / timeout failures that a second attempt might survive.
fn is_transient(error: &reqwest::Error) -> bool {
    error.is_timeout() || error.is_connect() || error.is_request()
}

/// The `Retry-After` header, in seconds, clamped to at most `MAX_RETRY_AFTER_MS`.
fn parse_retry_after(headers: &reqwest::header::HeaderMap) -> Option<Duration> {
    let raw = headers.get(reqwest::header::RETRY_AFTER)?.to_str().ok()?;
    let seconds: f64 = raw.trim().parse().ok()?;
    if seconds.is_finite() && seconds >= 0.0 {
        let millis = (seconds * 1000.0) as u64;
        Some(Duration::from_millis(millis.min(MAX_RETRY_AFTER_MS)))
    } else {
        None
    }
}

/// Exponential backoff for `attempt` (1-based), never shorter than the
/// server-requested `Retry-After` and never longer than `MAX_RETRY_AFTER_MS`.
fn backoff_delay(attempt: u32, retry_after: Option<Duration>) -> Duration {
    let step = 1u64 << (attempt.saturating_sub(1)).min(4); // 1,2,4,8,16
    let exponential = (BASE_BACKOFF_MS * step).min(MAX_BACKOFF_MS);
    let requested = retry_after.map(|d| d.as_millis() as u64).unwrap_or(0);
    Duration::from_millis(requested.min(MAX_RETRY_AFTER_MS).max(exponential))
}

/// Turn a transport failure into a friendly, non-fatal error.
fn map_transport_error(error: reqwest::Error) -> AppError {
    if error.is_timeout() {
        AppError::Ai(
            "The AI request timed out. Check your connection and try again.".to_string(),
        )
    } else {
        // Connect/request failures are almost always "no network".
        AppError::Offline
    }
}

/// Map an HTTP status to a plain-language error, keeping the provider's own
/// message (trimmed) so genuinely useful detail is not lost.
fn status_error(status: reqwest::StatusCode, provider_message: &str) -> AppError {
    let code = status.as_u16();
    let detail = provider_message.trim();
    let detail = if detail.len() > 300 {
        format!("{}…", &detail[..300])
    } else {
        detail.to_string()
    };

    let base = match code {
        401 | 403 => "The AI provider rejected the credentials (HTTP 401/403). Re-check the API key or access token in the assistant settings.",
        404 => "The model or base URL was not found (HTTP 404). Re-check the model id and base URL in the assistant settings.",
        413 => "The request was too large for the provider (HTTP 413). Try a shorter message.",
        429 => "The free-tier rate limit was reached (HTTP 429). Wait a few seconds and try again.",
        _ if status.is_server_error() => "The AI provider had a temporary server error. Try again in a moment.",
        400 if mentions_context_limit(&detail) => "The conversation is too long for this model. Try starting a new chat.",
        _ => "The AI provider returned an error.",
    };

    if detail.is_empty() {
        AppError::Ai(format!("{base} (HTTP {code})"))
    } else {
        AppError::Ai(format!("{base} (HTTP {code}: {detail})"))
    }
}

fn mentions_context_limit(message: &str) -> bool {
    let lower = message.to_lowercase();
    (lower.contains("context") || lower.contains("token"))
        && (lower.contains("length")
            || lower.contains("limit")
            || lower.contains("too long")
            || lower.contains("maximum"))
}

/// One tool call as it is reassembled from streamed deltas.
#[derive(Default)]
struct PartialToolCall {
    id: String,
    name: String,
    /// The arguments arrive as a JSON string split across many deltas.
    arguments: String,
}

/// Reassembles an OpenAI SSE stream into a single [`ProviderResponse`],
/// emitting each chunk of visible text as it arrives. Pure and side-effect
/// free, so it is unit-tested without a network.
#[derive(Default)]
struct StreamAssembler {
    /// Bytes not yet terminated by a newline (chunks can split lines).
    buffer: String,
    text: String,
    tool_calls: Vec<PartialToolCall>,
    stop_reason: Option<String>,
}

impl StreamAssembler {
    /// Feed one raw chunk; returns any visible text deltas it contained.
    fn push(&mut self, chunk: &str) -> Vec<String> {
        self.buffer.push_str(chunk);
        let mut deltas = Vec::new();

        while let Some(newline) = self.buffer.find('\n') {
            let line = self.buffer[..newline].trim_end_matches('\r').to_string();
            self.buffer.drain(..=newline);

            let Some(payload) = line.strip_prefix("data:") else {
                continue; // comment, event type, or blank line
            };
            let payload = payload.trim();
            if payload == "[DONE]" || payload.is_empty() {
                continue;
            }
            if let Some(delta) = self.consume(payload) {
                deltas.push(delta);
            }
        }

        deltas
    }

    fn consume(&mut self, payload: &str) -> Option<String> {
        let value: Value = serde_json::from_str(payload).ok()?;
        let choice = value.pointer("/choices/0")?;

        if let Some(reason) = choice.get("finish_reason").and_then(Value::as_str) {
            self.stop_reason = Some(reason.to_string());
        }

        let delta = choice.get("delta")?;

        if let Some(text) = delta.get("content").and_then(Value::as_str) {
            self.text.push_str(text);
            if !text.is_empty() {
                return Some(text.to_string());
            }
        }

        if let Some(calls) = delta.get("tool_calls").and_then(Value::as_array) {
            for call in calls {
                let index = call.get("index").and_then(Value::as_u64).unwrap_or(0) as usize;
                while self.tool_calls.len() <= index {
                    self.tool_calls.push(PartialToolCall::default());
                }
                let slot = &mut self.tool_calls[index];
                if let Some(id) = call.get("id").and_then(Value::as_str) {
                    slot.id = id.to_string();
                }
                if let Some(function) = call.get("function") {
                    if let Some(name) = function.get("name").and_then(Value::as_str) {
                        slot.name.push_str(name);
                    }
                    if let Some(args) = function.get("arguments").and_then(Value::as_str) {
                        slot.arguments.push_str(args);
                    }
                }
            }
        }

        None
    }

    fn finish(self) -> Result<ProviderResponse, AppError> {
        let mut content = Vec::new();
        if !self.text.is_empty() {
            content.push(ContentBlock::Text(self.text));
        }
        for call in self.tool_calls {
            if call.name.is_empty() {
                continue;
            }
            content.push(ContentBlock::ToolUse {
                id: call.id,
                name: call.name,
                input: serde_json::from_str(&call.arguments).unwrap_or_else(|_| json!({})),
            });
        }
        Ok(ProviderResponse {
            content,
            stop_reason: self.stop_reason,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn provider() -> OpenAiProvider {
        OpenAiProvider::new(
            Some("sk-test".into()),
            "http://localhost:11434/v1/".into(),
            "openai/gpt-oss-20b".into(),
        )
        .unwrap()
    }

    #[test]
    fn endpoint_joins_without_doubling_the_slash() {
        assert_eq!(
            provider().endpoint(),
            "http://localhost:11434/v1/chat/completions"
        );
    }

    #[test]
    fn body_leads_with_the_system_prompt_and_wraps_tools() {
        let request = ProviderRequest {
            system: "You are JARVIS.".into(),
            messages: vec![ProviderMessage::text("user", "Plan my day")],
            tools: vec![json!({
                "name": "create_task",
                "description": "Create a task",
                "input_schema": { "type": "object", "properties": { "title": { "type": "string" } }, "required": ["title"] }
            })],
            max_tokens: 1024,
        };

        let body = provider().body(&request, false);
        assert_eq!(body["model"], "openai/gpt-oss-20b");
        assert_eq!(body["max_tokens"], 1024);
        assert_eq!(body["messages"][0]["role"], "system");
        assert_eq!(body["messages"][0]["content"], "You are JARVIS.");
        assert_eq!(body["messages"][1]["role"], "user");
        assert_eq!(body["messages"][1]["content"], "Plan my day");
        assert!(body.get("stream").is_none());

        assert_eq!(body["tools"][0]["type"], "function");
        assert_eq!(body["tools"][0]["function"]["name"], "create_task");
        assert_eq!(
            body["tools"][0]["function"]["parameters"]["required"][0],
            "title"
        );
    }

    #[test]
    fn streaming_body_sets_the_stream_flag() {
        let request = ProviderRequest {
            system: "sys".into(),
            messages: vec![ProviderMessage::text("user", "hi")],
            tools: vec![],
            max_tokens: 256,
        };
        assert_eq!(provider().body(&request, true)["stream"], true);
    }

    #[test]
    fn assistant_tool_use_becomes_tool_calls_with_string_arguments() {
        let message = ProviderMessage::assistant(vec![
            ContentBlock::text("On it."),
            ContentBlock::ToolUse {
                id: "call_1".into(),
                name: "create_task".into(),
                input: json!({ "title": "Write report" }),
            },
        ]);

        let out = translate_message(&message);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0]["role"], "assistant");
        assert_eq!(out[0]["content"], "On it.");
        assert_eq!(out[0]["tool_calls"][0]["id"], "call_1");
        assert_eq!(out[0]["tool_calls"][0]["type"], "function");
        assert_eq!(out[0]["tool_calls"][0]["function"]["name"], "create_task");
        let args: Value =
            serde_json::from_str(out[0]["tool_calls"][0]["function"]["arguments"].as_str().unwrap())
                .unwrap();
        assert_eq!(args["title"], "Write report");
    }

    #[test]
    fn tool_results_fan_out_into_tool_messages_and_mark_errors() {
        let message = ProviderMessage::user(vec![
            ContentBlock::ToolResult {
                tool_use_id: "call_1".into(),
                content: "Created task #1.".into(),
                is_error: false,
            },
            ContentBlock::ToolResult {
                tool_use_id: "call_2".into(),
                content: "No such task".into(),
                is_error: true,
            },
        ]);

        let out = translate_message(&message);
        assert_eq!(out.len(), 2);
        assert_eq!(out[0]["role"], "tool");
        assert_eq!(out[0]["tool_call_id"], "call_1");
        assert_eq!(out[0]["content"], "Created task #1.");
        assert_eq!(out[1]["tool_call_id"], "call_2");
        assert_eq!(out[1]["content"], "Error: No such task");
    }

    #[test]
    fn assistant_tool_call_without_text_uses_null_content() {
        let message = ProviderMessage::assistant(vec![ContentBlock::ToolUse {
            id: "call_9".into(),
            name: "complete_task".into(),
            input: json!({ "task_id": 3 }),
        }]);
        let out = translate_message(&message);
        assert!(out[0]["content"].is_null());
    }

    #[test]
    fn parses_text_and_tool_calls_from_a_completion() {
        let body = json!({
            "id": "chatcmpl-1",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "Creating that now.",
                    "tool_calls": [{
                        "id": "call_7",
                        "type": "function",
                        "function": { "name": "create_habit", "arguments": "{\"title\":\"Meditate\"}" }
                    }]
                },
                "finish_reason": "tool_calls"
            }]
        });

        let response = parse_response(body).unwrap();
        assert_eq!(response.text(), "Creating that now.");
        assert_eq!(response.stop_reason.as_deref(), Some("tool_calls"));

        let tools = response.tool_uses();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].0, "call_7");
        assert_eq!(tools[0].1, "create_habit");
        assert_eq!(tools[0].2["title"], "Meditate");
    }

    #[test]
    fn plain_text_completion_has_no_tool_uses() {
        let body = json!({
            "choices": [{ "message": { "role": "assistant", "content": "All done." }, "finish_reason": "stop" }]
        });
        let response = parse_response(body).unwrap();
        assert_eq!(response.text(), "All done.");
        assert!(response.tool_uses().is_empty());
    }

    #[test]
    fn missing_choices_is_an_error() {
        assert!(parse_response(json!({ "id": "x" })).is_err());
    }

    #[test]
    fn body_omits_tools_when_there_are_none() {
        let request = ProviderRequest {
            system: "sys".into(),
            messages: vec![ProviderMessage::text("user", "hi")],
            tools: vec![],
            max_tokens: 256,
        };
        assert!(provider().body(&request, false).get("tools").is_none());
    }

    // ---- retry / backoff --------------------------------------------------

    #[test]
    fn only_429_and_5xx_are_retryable() {
        assert!(is_retryable_status(reqwest::StatusCode::TOO_MANY_REQUESTS));
        assert!(is_retryable_status(reqwest::StatusCode::BAD_GATEWAY));
        assert!(!is_retryable_status(reqwest::StatusCode::UNAUTHORIZED));
        assert!(!is_retryable_status(reqwest::StatusCode::NOT_FOUND));
        assert!(!is_retryable_status(reqwest::StatusCode::OK));
    }

    #[test]
    fn backoff_grows_then_caps() {
        assert_eq!(backoff_delay(1, None), Duration::from_millis(500));
        assert_eq!(backoff_delay(2, None), Duration::from_millis(1000));
        assert_eq!(backoff_delay(3, None), Duration::from_millis(2000));
        assert_eq!(backoff_delay(9, None), Duration::from_millis(MAX_BACKOFF_MS));
    }

    #[test]
    fn retry_after_wins_when_longer_but_is_capped() {
        assert_eq!(
            backoff_delay(1, Some(Duration::from_secs(5))),
            Duration::from_secs(5)
        );
        assert_eq!(
            backoff_delay(1, Some(Duration::from_secs(999))),
            Duration::from_millis(MAX_RETRY_AFTER_MS)
        );
    }

    #[test]
    fn parses_retry_after_seconds() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(reqwest::header::RETRY_AFTER, "3".parse().unwrap());
        assert_eq!(parse_retry_after(&headers), Some(Duration::from_secs(3)));

        let mut bad = reqwest::header::HeaderMap::new();
        bad.insert(reqwest::header::RETRY_AFTER, "soon".parse().unwrap());
        assert_eq!(parse_retry_after(&bad), None);
    }

    #[test]
    fn status_errors_are_plain_language() {
        let message = |code| match status_error(reqwest::StatusCode::from_u16(code).unwrap(), "") {
            AppError::Ai(text) => text,
            other => panic!("expected Ai error, got {other}"),
        };
        assert!(message(401).contains("credentials"));
        assert!(message(404).contains("not found"));
        assert!(message(429).contains("rate limit"));
        assert!(message(500).contains("server error"));
    }

    #[test]
    fn context_limit_errors_are_recognised() {
        assert!(mentions_context_limit("This model's maximum context length is 8192 tokens"));
        assert!(!mentions_context_limit("invalid tool name"));
    }

    // ---- streaming --------------------------------------------------------

    #[test]
    fn assembles_streamed_text_deltas() {
        let mut assembler = StreamAssembler::default();
        let chunk = concat!(
            "data: {\"choices\":[{\"delta\":{\"content\":\"Hello\"}}]}\n\n",
            "data: {\"choices\":[{\"delta\":{\"content\":\" there\"}}]}\n\n",
            "data: [DONE]\n\n",
        );
        let deltas = assembler.push(chunk);
        assert_eq!(deltas, vec!["Hello".to_string(), " there".to_string()]);
        let response = assembler.finish().unwrap();
        assert_eq!(response.text(), "Hello there");
    }

    #[test]
    fn assembles_tool_calls_split_across_chunks() {
        let mut assembler = StreamAssembler::default();
        // The arguments string is split mid-JSON, exactly as OpenAI streams it.
        assembler.push("data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"id\":\"call_1\",\"function\":{\"name\":\"create_task\",\"arguments\":\"{\\\"title\\\":\"}}]}}]}\n");
        assembler.push("data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"function\":{\"arguments\":\"\\\"Call mum\\\"}\"}}]},\"finish_reason\":\"tool_calls\"}]}\n");
        assembler.push("data: [DONE]\n");

        let response = assembler.finish().unwrap();
        assert_eq!(response.stop_reason.as_deref(), Some("tool_calls"));
        let tools = response.tool_uses();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].1, "create_task");
        assert_eq!(tools[0].2["title"], "Call mum");
    }

    #[test]
    fn handles_lines_split_across_network_chunks() {
        let mut assembler = StreamAssembler::default();
        assert!(assembler.push("data: {\"choices\":[{\"delta\":{\"cont").is_empty());
        let deltas = assembler.push("ent\":\"hi\"}}]}\n");
        assert_eq!(deltas, vec!["hi".to_string()]);
    }
}
