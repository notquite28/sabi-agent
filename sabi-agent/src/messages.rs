//! Message and content data types shared by the agent loop, tools, and sessions.
//!
//! Ported from:
//! - `pi/packages/ai/src/types.ts`
//! - `pi/packages/agent/src/types.ts`
//! - `pi/packages/agent/src/harness/messages.ts`
//!
//! Simplifications:
//! - Starts text-only.
//! - Does not include images, thinking blocks, custom UI messages, or provider diagnostics yet.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "role")]
pub enum Message {
    #[serde(rename = "user")]
    User { content: String },
    #[serde(rename = "assistant")]
    Assistant {
        content: String,
        #[serde(default)]
        tool_calls: Vec<ToolCall>,
    },
    #[serde(rename = "tool")]
    ToolResult {
        tool_call_id: String,
        tool_name: String,
        content: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

impl Message {
    pub fn user(content: impl Into<String>) -> Self {
        Self::User {
            content: content.into(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self::Assistant {
            content: content.into(),
            tool_calls: Vec::new(),
        }
    }

    pub fn assistant_with_tool_calls(
        content: impl Into<String>,
        tool_calls: Vec<ToolCall>,
    ) -> Self {
        Self::Assistant {
            content: content.into(),
            tool_calls,
        }
    }

    pub fn tool_result(
        tool_call_id: impl Into<String>,
        tool_name: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        Self::ToolResult {
            tool_call_id: tool_call_id.into(),
            tool_name: tool_name.into(),
            content: content.into(),
        }
    }

    pub fn role(&self) -> &'static str {
        match self {
            Self::User { .. } => "user",
            Self::Assistant { .. } => "assistant",
            Self::ToolResult { .. } => "tool",
        }
    }

    pub fn content(&self) -> &str {
        match self {
            Self::User { content } => content,
            Self::Assistant { content, .. } => content,
            Self::ToolResult { content, .. } => content,
        }
    }

    pub fn tool_calls(&self) -> &[ToolCall] {
        match self {
            Self::Assistant { tool_calls, .. } => tool_calls,
            _ => &[],
        }
    }
}
