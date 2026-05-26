//! Directory listing tool implementation.
//!
//! Ported from:
//! - `pi/packages/coding-agent/src/core/tools/ls.ts`
//!
//! Simplifications:
//! - Sorted local filesystem entries only.
//! - No custom remote operations or TUI rendering yet.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::{json, Value};

use super::{object_schema, ToolSpec};

#[derive(Debug, Deserialize)]
struct LsArgs {
    path: Option<String>,
}

pub fn spec() -> ToolSpec {
    ToolSpec {
        name: "ls",
        description: "List files and directories in a directory. Prefer this over bash ls.",
        parameters: object_schema(
            json!({
                "path": { "type": "string", "description": "Directory path to list. Defaults to current working directory." }
            }),
            vec![],
        ),
    }
}

pub async fn run(args: Value, cwd: &Path) -> Result<String> {
    let args: LsArgs = serde_json::from_value(args).context("invalid ls arguments")?;
    let path = resolve_path(cwd, args.path.as_deref().unwrap_or("."));
    let mut entries = tokio::fs::read_dir(&path)
        .await
        .with_context(|| format!("failed to list {}", path.display()))?;
    let mut names = Vec::new();

    while let Some(entry) = entries.next_entry().await? {
        let file_type = entry.file_type().await?;
        let mut name = entry.file_name().to_string_lossy().to_string();
        if file_type.is_dir() {
            name.push('/');
        }
        names.push(name);
    }

    names.sort();
    if names.is_empty() {
        Ok(format!("{} is empty", path.display()))
    } else {
        Ok(names.join("\n"))
    }
}

fn resolve_path(cwd: &Path, path: &str) -> PathBuf {
    let path = PathBuf::from(path);
    if path.is_absolute() {
        path
    } else {
        cwd.join(path)
    }
}
