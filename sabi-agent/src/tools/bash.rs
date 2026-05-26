//! Bash tool implementation.
//!
//! Ported from:
//! - `pi/packages/coding-agent/src/core/tools/bash.ts`
//! - `pi/packages/coding-agent/src/core/bash-executor.ts`
//!
//! Simplifications:
//! - Starts with direct shell command execution and timeout.
//! - No partial output updates, process-tree tracking, or custom shell settings yet.

use std::path::Path;
use std::time::Duration;

use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::process::Command;

use super::{object_schema, ToolSpec};

#[derive(Debug, Deserialize)]
struct BashArgs {
    command: String,
    timeout: Option<u64>,
}

pub fn spec() -> ToolSpec {
    ToolSpec {
        name: "bash",
        description: "Run a shell command in the current working directory and return combined stdout/stderr.",
        parameters: object_schema(
            json!({
                "command": { "type": "string", "description": "Shell command to run" },
                "timeout": { "type": "integer", "description": "Timeout in seconds" }
            }),
            vec!["command"],
        ),
    }
}

pub async fn run(args: Value, cwd: &Path) -> Result<String> {
    let args: BashArgs = serde_json::from_value(args).context("invalid bash arguments")?;
    let mut child = Command::new("sh");
    child.arg("-c").arg(&args.command).current_dir(cwd);

    let output_future = child.output();
    let output = if let Some(seconds) = args.timeout {
        tokio::time::timeout(Duration::from_secs(seconds), output_future)
            .await
            .with_context(|| format!("command timed out after {seconds}s"))??
    } else {
        output_future.await?
    };

    let mut text = String::new();
    if !output.stdout.is_empty() {
        text.push_str(&String::from_utf8_lossy(&output.stdout));
    }
    if !output.stderr.is_empty() {
        if !text.is_empty() {
            text.push('\n');
        }
        text.push_str(&String::from_utf8_lossy(&output.stderr));
    }
    if text.trim().is_empty() {
        text = format!("exit status: {}", output.status);
    } else {
        text.push_str(&format!("\n\n[exit status: {}]", output.status));
    }

    Ok(text)
}
