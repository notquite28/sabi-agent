//! Tool registry and shared tool types.
//!
//! Ported from:
//! - `pi/packages/coding-agent/src/core/tools/index.ts`
//! - `pi/packages/agent/src/types.ts`
//!
//! Simplifications:
//! - Starts with a small static set of built-in tools.
//! - No extension-registered tools, SDK overrides, or custom render components yet.

pub mod bash;
pub mod edit;
pub mod find;
pub mod grep;
pub mod ls;
pub mod read;
pub mod write;

use std::path::Path;

use anyhow::Result;
use serde_json::{json, Value};

#[derive(Debug, Clone)]
pub struct ToolOutput {
    pub content: String,
    pub is_error: bool,
}

#[derive(Debug, Clone)]
pub struct ToolSpec {
    pub name: &'static str,
    pub description: &'static str,
    pub parameters: Value,
}

pub fn builtin_tool_specs() -> Vec<ToolSpec> {
    vec![read::spec(), write::spec(), bash::spec()]
}

pub async fn run_tool(name: &str, args: Value, cwd: &Path) -> ToolOutput {
    let result: Result<String> = match name {
        "read" => read::run(args, cwd).await,
        "write" => write::run(args, cwd).await,
        "bash" => bash::run(args, cwd).await,
        _ => Err(anyhow::anyhow!("unknown tool: {name}")),
    };

    match result {
        Ok(content) => ToolOutput {
            content,
            is_error: false,
        },
        Err(error) => ToolOutput {
            content: error.to_string(),
            is_error: true,
        },
    }
}

fn object_schema(properties: Value, required: Vec<&str>) -> Value {
    json!({
        "type": "object",
        "properties": properties,
        "required": required,
        "additionalProperties": false
    })
}
