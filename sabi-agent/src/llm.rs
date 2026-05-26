//! Provider HTTP integration.
//!
//! Ported from:
//! - `pi/packages/ai/src/providers/openai-responses.ts`
//! - `pi/packages/ai/src/stream.ts`
//! - `pi/packages/ai/src/types.ts`
//!
//! Simplifications:
//! - The first implementation will support one OpenAI-compatible provider.
//! - No provider registry, OAuth, prompt caching, images, or provider-specific compatibility layer yet.

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::messages::{Message, ToolCall};
use crate::tools::ToolSpec;

#[derive(Debug, Clone)]
pub struct ModelConfig {
    pub model: String,
    pub base_url: String,
    pub api_key: String,
}

#[derive(Debug, Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<ChatTool<'a>>>,
}

#[derive(Debug, Serialize)]
struct ChatMessage {
    role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<OpenAiToolCall>>,
}

#[derive(Debug, Serialize)]
struct ChatTool<'a> {
    #[serde(rename = "type")]
    kind: &'static str,
    function: ChatToolFunction<'a>,
}

#[derive(Debug, Serialize)]
struct ChatToolFunction<'a> {
    name: &'a str,
    description: &'a str,
    parameters: &'a Value,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatResponseMessage,
}

#[derive(Debug, Deserialize)]
struct ChatResponseMessage {
    content: Option<String>,
    #[serde(default)]
    tool_calls: Option<Vec<OpenAiToolCall>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAiToolCall {
    id: String,
    #[serde(rename = "type")]
    kind: String,
    function: OpenAiToolCallFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAiToolCallFunction {
    name: String,
    arguments: String,
}

#[derive(Debug, Deserialize)]
struct ErrorResponse {
    error: ErrorBody,
}

#[derive(Debug, Deserialize)]
struct ErrorBody {
    message: String,
}

#[derive(Debug, Deserialize)]
struct ModelsResponse {
    data: Vec<ModelInfo>,
}

#[derive(Debug, Deserialize)]
struct ModelInfo {
    id: String,
    #[serde(default)]
    supported_endpoint_types: Vec<String>,
}

pub async fn check_provider(config: &ModelConfig) -> Result<()> {
    println!("base URL: {}", config.base_url);
    println!("selected model: {}", config.model);

    let models = list_models(config).await?;
    println!("models endpoint: ok ({} models)", models.len());

    let selected = models.iter().find(|model| model.id == config.model);
    match selected {
        Some(model) => {
            if model.supported_endpoint_types.is_empty() {
                println!("selected model: found");
            } else {
                println!(
                    "selected model: found, supported endpoints: {}",
                    model.supported_endpoint_types.join(", ")
                );
            }
        }
        None => {
            println!("selected model: not found in /models");
            println!("available models:");
            for model in &models {
                println!("- {}", model.id);
            }
            anyhow::bail!("selected model is not available to this API key");
        }
    }

    let messages = vec![Message::user("Say exactly: ok")];
    let reply = complete_chat(config, &messages).await?;
    println!("chat completions endpoint: ok");
    println!("test reply: {reply}");

    Ok(())
}

async fn list_models(config: &ModelConfig) -> Result<Vec<ModelInfo>> {
    let url = format!("{}/models", config.base_url.trim_end_matches('/'));
    let response = Client::new()
        .get(url)
        .bearer_auth(&config.api_key)
        .send()
        .await
        .context("failed to send models request")?;

    let status = response.status();
    let bytes = response
        .bytes()
        .await
        .context("failed to read models response")?;

    if !status.is_success() {
        if let Ok(error) = serde_json::from_slice::<ErrorResponse>(&bytes) {
            anyhow::bail!("models endpoint error ({status}): {}", error.error.message);
        }
        let body = String::from_utf8_lossy(&bytes);
        anyhow::bail!("models endpoint error ({status}): {body}");
    }

    let response: ModelsResponse =
        serde_json::from_slice(&bytes).context("failed to parse models response")?;
    Ok(response.data)
}

pub async fn complete_chat(config: &ModelConfig, messages: &[Message]) -> Result<String> {
    let message = complete_chat_message(config, messages, &[]).await?;
    Ok(message.content().to_string())
}

pub async fn complete_chat_message(
    config: &ModelConfig,
    messages: &[Message],
    tools: &[ToolSpec],
) -> Result<Message> {
    let url = format!("{}/chat/completions", config.base_url.trim_end_matches('/'));
    let request = ChatRequest {
        model: &config.model,
        messages: messages.iter().map(to_chat_message).collect(),
        tools: if tools.is_empty() {
            None
        } else {
            Some(
                tools
                    .iter()
                    .map(|tool| ChatTool {
                        kind: "function",
                        function: ChatToolFunction {
                            name: tool.name,
                            description: tool.description,
                            parameters: &tool.parameters,
                        },
                    })
                    .collect(),
            )
        },
    };

    let response = Client::new()
        .post(url)
        .bearer_auth(&config.api_key)
        .json(&request)
        .send()
        .await
        .context("failed to send provider request")?;

    let status = response.status();
    let bytes = response
        .bytes()
        .await
        .context("failed to read provider response")?;

    if !status.is_success() {
        if let Ok(error) = serde_json::from_slice::<ErrorResponse>(&bytes) {
            anyhow::bail!("provider error ({status}): {}", error.error.message);
        }
        let body = String::from_utf8_lossy(&bytes);
        anyhow::bail!("provider error ({status}): {body}");
    }

    let response: ChatResponse =
        serde_json::from_slice(&bytes).context("failed to parse provider response")?;
    let message = response
        .choices
        .into_iter()
        .next()
        .map(|choice| choice.message)
        .context("provider response did not include a choice")?;
    let tool_calls = message
        .tool_calls
        .unwrap_or_default()
        .into_iter()
        .map(|tool_call| ToolCall {
            id: tool_call.id,
            name: tool_call.function.name,
            arguments: tool_call.function.arguments,
        })
        .collect();

    Ok(Message::assistant_with_tool_calls(
        message.content.unwrap_or_default(),
        tool_calls,
    ))
}

fn to_chat_message(message: &Message) -> ChatMessage {
    match message {
        Message::User { content } => ChatMessage {
            role: "user".to_string(),
            content: Some(content.clone()),
            tool_call_id: None,
            tool_calls: None,
        },
        Message::Assistant {
            content,
            tool_calls,
        } => ChatMessage {
            role: "assistant".to_string(),
            content: if content.is_empty() {
                None
            } else {
                Some(content.clone())
            },
            tool_call_id: None,
            tool_calls: if tool_calls.is_empty() {
                None
            } else {
                Some(
                    tool_calls
                        .iter()
                        .map(|tool_call| OpenAiToolCall {
                            id: tool_call.id.clone(),
                            kind: "function".to_string(),
                            function: OpenAiToolCallFunction {
                                name: tool_call.name.clone(),
                                arguments: tool_call.arguments.clone(),
                            },
                        })
                        .collect(),
                )
            },
        },
        Message::ToolResult {
            tool_call_id,
            content,
            ..
        } => ChatMessage {
            role: "tool".to_string(),
            content: Some(content.clone()),
            tool_call_id: Some(tool_call_id.clone()),
            tool_calls: None,
        },
    }
}
