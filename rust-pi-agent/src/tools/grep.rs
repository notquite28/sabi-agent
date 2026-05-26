//! Grep tool implementation.
//!
//! Ported from:
//! - `pi/packages/coding-agent/src/core/tools/grep.ts`
//!
//! Simplifications:
//! - Uses installed `rg` instead of downloading or embedding ripgrep.
//! - No custom remote operations or JSON event streaming cache yet.

use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::process::Command;

use super::{object_schema, ToolSpec};

#[derive(Debug, Deserialize)]
struct GrepArgs {
    pattern: String,
    path: Option<String>,
    include: Option<String>,
    limit: Option<usize>,
}

pub fn spec() -> ToolSpec {
    ToolSpec {
        name: "grep",
        description: "Search file contents using ripgrep. Prefer this over bash grep or rg.",
        parameters: object_schema(
            json!({
                "pattern": { "type": "string", "description": "Regular expression to search for" },
                "path": { "type": "string", "description": "Directory or file to search. Defaults to current working directory." },
                "include": { "type": "string", "description": "Glob of files to include, such as *.rs" },
                "limit": { "type": "integer", "description": "Maximum number of matching lines to return. Defaults to 200." }
            }),
            vec!["pattern"],
        ),
    }
}

pub async fn run(args: Value, cwd: &Path) -> Result<String> {
    let args: GrepArgs = serde_json::from_value(args).context("invalid grep arguments")?;
    let path = args.path.unwrap_or_else(|| ".".to_string());
    let limit = args.limit.unwrap_or(200);

    let mut command = Command::new("rg");
    command.arg("--line-number").arg("--color=never");
    if let Some(include) = args.include {
        command.arg("--glob").arg(include);
    }
    command.arg(&args.pattern).arg(&path).current_dir(cwd);

    let output = command
        .output()
        .await
        .context("failed to run rg; install ripgrep or use bash as a fallback")?;

    if output.status.code() == Some(1) {
        return Ok("no matches found".to_string());
    }
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("rg failed: {stderr}");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut lines: Vec<&str> = stdout.lines().collect();
    let total = lines.len();
    if lines.len() > limit {
        lines.truncate(limit);
    }

    let mut result = lines.join("\n");
    if total > limit {
        result.push_str(&format!(
            "\n\n[truncated: showing {limit} of {total} matches]"
        ));
    }
    Ok(result)
}
