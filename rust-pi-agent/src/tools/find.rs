//! File finder tool implementation.
//!
//! Ported from:
//! - `pi/packages/coding-agent/src/core/tools/find.ts`
//!
//! Simplifications:
//! - Uses installed `fd` at first.
//! - No automatic tool download or custom remote glob operations yet.

use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::process::Command;

use super::{object_schema, ToolSpec};

#[derive(Debug, Deserialize)]
struct FindArgs {
    pattern: Option<String>,
    path: Option<String>,
    limit: Option<usize>,
}

pub fn spec() -> ToolSpec {
    ToolSpec {
        name: "find",
        description: "Find files by name pattern using fd. Prefer this over bash find.",
        parameters: object_schema(
            json!({
                "pattern": { "type": "string", "description": "Name pattern to search for. Defaults to all files." },
                "path": { "type": "string", "description": "Directory to search. Defaults to current working directory." },
                "limit": { "type": "integer", "description": "Maximum number of paths to return. Defaults to 200." }
            }),
            vec![],
        ),
    }
}

pub async fn run(args: Value, cwd: &Path) -> Result<String> {
    let args: FindArgs = serde_json::from_value(args).context("invalid find arguments")?;
    let pattern = args.pattern.unwrap_or_else(|| ".".to_string());
    let search_path = args.path.unwrap_or_else(|| ".".to_string());
    let limit = args.limit.unwrap_or(200);

    let output = Command::new("fd")
        .arg(&pattern)
        .arg(&search_path)
        .current_dir(cwd)
        .output()
        .await
        .context("failed to run fd; install fd or use bash as a fallback")?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        anyhow::bail!("fd failed: {stderr}");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut lines: Vec<&str> = stdout.lines().collect();
    let total = lines.len();
    if lines.len() > limit {
        lines.truncate(limit);
    }

    if lines.is_empty() {
        return Ok("no files found".to_string());
    }

    let mut result = lines.join("\n");
    if total > limit {
        result.push_str(&format!(
            "\n\n[truncated: showing {limit} of {total} paths]"
        ));
    }
    Ok(result)
}
