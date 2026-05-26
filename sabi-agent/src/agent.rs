//! Agent loop and tool-call orchestration.
//!
//! Ported from:
//! - `pi/packages/agent/src/agent-loop.ts`
//! - `pi/packages/agent/src/agent.ts`
//! - `pi/packages/agent/src/types.ts`
//!
//! Simplifications:
//! - Will start with a linear loop: assistant response, tool execution, tool results, repeat.
//! - No steering queue, follow-up queue, extension hooks, compaction, or branching yet.

use std::path::Path;

use anyhow::{Context, Result};
use serde_json::Value;

use crate::events::AgentEvent;
use crate::llm::{complete_chat_message, ModelConfig};
use crate::messages::Message;
use crate::session::SessionStore;
use crate::tools::{builtin_tool_specs, run_tool, ToolOutput};

#[derive(Debug, Default)]
pub struct AgentState {
    pub messages_len: usize,
}

const MAX_TOOL_ROUNDS: usize = 8;

pub async fn run_agent_turn(
    model: &ModelConfig,
    messages: &mut Vec<Message>,
    cwd: &Path,
    prompt: &str,
) -> Result<String> {
    run_agent_turn_with_events(model, messages, cwd, prompt, None, |_| {}, |_, _| true).await
}

pub async fn run_agent_turn_with_events(
    model: &ModelConfig,
    messages: &mut Vec<Message>,
    cwd: &Path,
    prompt: &str,
    session: Option<&SessionStore>,
    mut emit: impl FnMut(AgentEvent),
    mut approve: impl FnMut(&str, &Value) -> bool,
) -> Result<String> {
    let user_message = Message::user(prompt);
    persist_message(session, &user_message).await?;
    messages.push(user_message);
    let tools = builtin_tool_specs();

    for _round in 0..MAX_TOOL_ROUNDS {
        let assistant = complete_chat_message(model, messages, &tools).await?;
        let text = assistant.content().to_string();
        let tool_calls = assistant.tool_calls().to_vec();

        if !text.trim().is_empty() {
            emit(AgentEvent::AssistantText { text: text.clone() });
        }

        persist_message(session, &assistant).await?;
        messages.push(assistant);

        if tool_calls.is_empty() {
            return Ok(text);
        }

        for tool_call in tool_calls {
            let args: Value = serde_json::from_str(&tool_call.arguments)
                .with_context(|| format!("invalid JSON arguments for {}", tool_call.name))?;
            emit(AgentEvent::ToolStarted {
                id: tool_call.id.clone(),
                name: tool_call.name.clone(),
                args: args.clone(),
            });
            let output = if approve(&tool_call.name, &args) {
                run_tool(&tool_call.name, args, cwd).await
            } else {
                ToolOutput {
                    content: format!("tool execution denied by user: {}", tool_call.name),
                    is_error: true,
                    events: Vec::new(),
                }
            };
            for event in output.events.clone() {
                emit(event);
            }
            emit(AgentEvent::ToolFinished {
                id: tool_call.id.clone(),
                name: tool_call.name.clone(),
                output: output.content.clone(),
                is_error: output.is_error,
            });
            let tool_result = Message::tool_result(tool_call.id, tool_call.name, output.content);
            persist_message(session, &tool_result).await?;
            messages.push(tool_result);
        }
    }

    anyhow::bail!("agent stopped after {MAX_TOOL_ROUNDS} tool rounds to avoid an infinite loop")
}

async fn persist_message(session: Option<&SessionStore>, message: &Message) -> Result<()> {
    if let Some(session) = session {
        session.append_message(message).await?;
    }
    Ok(())
}

pub fn first_line(text: &str) -> &str {
    text.lines().next().unwrap_or("")
}
