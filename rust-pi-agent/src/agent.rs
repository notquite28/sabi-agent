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

use crate::llm::{complete_chat_message, ModelConfig};
use crate::messages::Message;
use crate::tools::{builtin_tool_specs, run_tool};

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
    messages.push(Message::user(prompt));
    let tools = builtin_tool_specs();
    let mut final_text = String::new();

    for _round in 0..MAX_TOOL_ROUNDS {
        let assistant = complete_chat_message(model, messages, &tools).await?;
        let text = assistant.content().to_string();
        let tool_calls = assistant.tool_calls().to_vec();

        if !text.trim().is_empty() {
            println!("\n{text}\n");
            final_text = text;
        }

        messages.push(assistant);

        if tool_calls.is_empty() {
            return Ok(final_text);
        }

        for tool_call in tool_calls {
            println!("tool: {} {}", tool_call.name, tool_call.arguments);
            let args: Value = serde_json::from_str(&tool_call.arguments)
                .with_context(|| format!("invalid JSON arguments for {}", tool_call.name))?;
            let output = run_tool(&tool_call.name, args, cwd).await;
            if output.is_error {
                println!("tool error: {}", output.content);
            } else {
                println!("tool result: {}", first_line(&output.content));
            }
            messages.push(Message::tool_result(
                tool_call.id,
                tool_call.name,
                output.content,
            ));
        }
    }

    anyhow::bail!("agent stopped after {MAX_TOOL_ROUNDS} tool rounds to avoid an infinite loop")
}

fn first_line(text: &str) -> &str {
    text.lines().next().unwrap_or("")
}
