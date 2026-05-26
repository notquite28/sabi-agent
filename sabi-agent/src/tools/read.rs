//! Read tool implementation.
//!
//! Ported from:
//! - `pi/packages/coding-agent/src/core/tools/read.ts`
//!
//! Simplifications:
//! - Text files only at first.
//! - No image attachments, syntax highlighting, TUI rendering, or model vision checks.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::{json, Value};

use super::{object_schema, ToolSpec};

#[derive(Debug, Deserialize)]
struct ReadArgs {
    path: String,
    offset: Option<usize>,
    limit: Option<usize>,
}

pub fn spec() -> ToolSpec {
    ToolSpec {
        name: "read",
        description: "Read a UTF-8 text file. Supports optional 1-indexed offset and line limit.",
        parameters: object_schema(
            json!({
                "path": { "type": "string", "description": "Path to the file to read" },
                "offset": { "type": "integer", "description": "1-indexed line number to start from" },
                "limit": { "type": "integer", "description": "Maximum number of lines to return" }
            }),
            vec!["path"],
        ),
    }
}

pub async fn run(args: Value, cwd: &Path) -> Result<String> {
    let args: ReadArgs = serde_json::from_value(args).context("invalid read arguments")?;
    let path = resolve_path(cwd, &args.path);
    let content = tokio::fs::read_to_string(&path)
        .await
        .with_context(|| format!("failed to read {}", path.display()))?;

    let lines: Vec<&str> = content.lines().collect();
    let start = args.offset.unwrap_or(1).saturating_sub(1);
    let limit = args.limit.unwrap_or(200);
    let end = usize::min(start + limit, lines.len());

    if start >= lines.len() {
        return Ok(String::new());
    }

    let mut output = String::new();
    for (index, line) in lines[start..end].iter().enumerate() {
        let line_number = start + index + 1;
        output.push_str(&format!("{line_number}: {line}\n"));
    }
    if end < lines.len() {
        output.push_str(&format!(
            "\n[truncated: showing lines {}-{} of {}]",
            start + 1,
            end,
            lines.len()
        ));
    }

    Ok(output)
}

fn resolve_path(cwd: &Path, path: &str) -> PathBuf {
    let path = PathBuf::from(path);
    if path.is_absolute() {
        path
    } else {
        cwd.join(path)
    }
}
