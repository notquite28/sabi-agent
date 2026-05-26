//! Edit tool implementation.
//!
//! Ported from:
//! - `pi/packages/coding-agent/src/core/tools/edit.ts`
//! - `pi/packages/coding-agent/src/core/tools/edit-diff.ts`
//!
//! Simplifications:
//! - Starts with exact text replacement only.
//! - No fuzzy Unicode normalization, preview component, or TUI box rendering.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::diff::{render_terminal_diff, unified_patch};
use crate::events::AgentEvent;

use super::{object_schema, ToolOutput, ToolSpec};

#[derive(Debug, Deserialize)]
struct EditArgs {
    path: String,
    old_text: String,
    new_text: String,
}

pub fn spec() -> ToolSpec {
    ToolSpec {
        name: "edit",
        description: "Edit a UTF-8 text file by replacing one exact, unique text snippet.",
        parameters: object_schema(
            json!({
                "path": { "type": "string", "description": "Path to the file to edit" },
                "old_text": { "type": "string", "description": "Exact text to replace. Must appear exactly once." },
                "new_text": { "type": "string", "description": "Replacement text" }
            }),
            vec!["path", "old_text", "new_text"],
        ),
    }
}

pub async fn run(args: Value, cwd: &Path) -> Result<ToolOutput> {
    let args: EditArgs = serde_json::from_value(args).context("invalid edit arguments")?;
    if args.old_text.is_empty() {
        anyhow::bail!("old_text must not be empty");
    }
    if args.old_text == args.new_text {
        anyhow::bail!("old_text and new_text are identical");
    }

    let path = resolve_path(cwd, &args.path);
    let old_content = tokio::fs::read_to_string(&path)
        .await
        .with_context(|| format!("failed to read {}", path.display()))?;

    let matches = old_content.matches(&args.old_text).count();
    match matches {
        0 => anyhow::bail!("old_text was not found in {}", path.display()),
        1 => {}
        _ => anyhow::bail!(
            "old_text appears {matches} times in {}; make it unique",
            path.display()
        ),
    }

    let new_content = old_content.replacen(&args.old_text, &args.new_text, 1);
    let patch = unified_patch(&args.path, &old_content, &new_content);
    let rendered = render_terminal_diff(&old_content, &new_content);

    tokio::fs::write(&path, new_content)
        .await
        .with_context(|| format!("failed to write {}", path.display()))?;

    let display_path = path.display().to_string();
    Ok(ToolOutput {
        content: format!("edited {display_path}"),
        is_error: false,
        events: vec![
            AgentEvent::DiffReady {
                path: display_path.clone(),
                patch,
                rendered,
            },
            AgentEvent::FileChanged { path: display_path },
        ],
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
