//! Write tool implementation.
//!
//! Ported from:
//! - `pi/packages/coding-agent/src/core/tools/write.ts`
//!
//! Simplifications:
//! - Creates parent directories and writes UTF-8 text.
//! - No TUI preview, syntax highlighting, or incremental render cache.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::{json, Value};

use super::{object_schema, ToolSpec};

#[derive(Debug, Deserialize)]
struct WriteArgs {
    path: String,
    content: String,
}

pub fn spec() -> ToolSpec {
    ToolSpec {
        name: "write",
        description:
            "Create or overwrite a UTF-8 text file. Parent directories are created automatically.",
        parameters: object_schema(
            json!({
                "path": { "type": "string", "description": "Path to write" },
                "content": { "type": "string", "description": "Complete file content" }
            }),
            vec!["path", "content"],
        ),
    }
}

pub async fn run(args: Value, cwd: &Path) -> Result<String> {
    let args: WriteArgs = serde_json::from_value(args).context("invalid write arguments")?;
    let path = resolve_path(cwd, &args.path);
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    tokio::fs::write(&path, args.content)
        .await
        .with_context(|| format!("failed to write {}", path.display()))?;
    Ok(format!("wrote {}", path.display()))
}

fn resolve_path(cwd: &Path, path: &str) -> PathBuf {
    let path = PathBuf::from(path);
    if path.is_absolute() {
        path
    } else {
        cwd.join(path)
    }
}
