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

use crate::diff::{render_terminal_diff, unified_patch};
use crate::events::AgentEvent;

use super::{object_schema, ToolOutput, ToolSpec};

#[derive(Debug, Deserialize)]
struct WriteArgs {
    path: String,
    content: String,
}

pub fn spec() -> ToolSpec {
    ToolSpec {
        name: "write",
        description:
            "Create or overwrite a UTF-8 text file. Prefer edit for small changes to existing files.",
        parameters: object_schema(
            json!({
                "path": { "type": "string", "description": "Path to write" },
                "content": { "type": "string", "description": "Complete file content" }
            }),
            vec!["path", "content"],
        ),
    }
}

pub async fn run(args: Value, cwd: &Path) -> Result<ToolOutput> {
    let args: WriteArgs = serde_json::from_value(args).context("invalid write arguments")?;
    let path = resolve_path(cwd, &args.path);
    let old_content = tokio::fs::read_to_string(&path).await.ok();

    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let new_content = args.content;
    tokio::fs::write(&path, &new_content)
        .await
        .with_context(|| format!("failed to write {}", path.display()))?;

    let display_path = path.display().to_string();
    let mut events = Vec::new();
    if let Some(old_content) = old_content {
        if old_content != new_content {
            events.push(AgentEvent::DiffReady {
                path: display_path.clone(),
                patch: unified_patch(&args.path, &old_content, &new_content),
                rendered: render_terminal_diff(&old_content, &new_content),
            });
        }
    }
    events.push(AgentEvent::FileChanged {
        path: display_path.clone(),
    });

    Ok(ToolOutput {
        content: format!("wrote {display_path}"),
        is_error: false,
        events,
    })
}

fn resolve_path(cwd: &Path, path: &str) -> PathBuf {
    let path = PathBuf::from(path);
    if path.is_absolute() {
        path
    } else {
        cwd.join(path)
    }
}
