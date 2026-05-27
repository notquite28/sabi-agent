//! Structured events emitted by the agent engine.
//!
//! Ported from:
//! - `pi/packages/agent/src/agent-loop.ts`
//! - `pi/packages/agent/src/types.ts`
//!
//! Simplifications:
//! - Starts with events needed by the current CLI renderer.
//! - No streaming deltas, approvals, custom UI payloads, or session events yet.

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentEvent {
    AssistantText {
        text: String,
    },
    ToolStarted {
        id: String,
        name: String,
        args: Value,
    },
    ToolFinished {
        id: String,
        name: String,
        output: String,
        is_error: bool,
    },
    DiffReady {
        path: String,
        patch: String,
        rendered: String,
    },
    FileChanged {
        path: String,
    },
    Error {
        message: String,
    },
}
