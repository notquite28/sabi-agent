//! Tauri command bridge for the Sabi Agent desktop frontend.
//!
//! Ported from:
//! - `sabi-agent/src/desktop.rs`
//!
//! Simplifications:
//! - Workspace selection is frontend state; commands accept explicit cwd strings.
//! - Prompt execution, event streaming, and approval responses are added after the shell is stable.

use std::path::PathBuf;

use sabi_agent::desktop::{DesktopAgent, DesktopSessionInfo};

#[derive(Debug, thiserror::Error)]
enum DesktopCommandError {
    #[error("failed to resolve current directory: {0}")]
    CurrentDir(#[from] std::io::Error),
    #[error("agent error: {0}")]
    Agent(#[from] anyhow::Error),
}

impl serde::Serialize for DesktopCommandError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[tauri::command]
fn health() -> &'static str {
    "ok"
}

#[tauri::command]
fn current_workspace() -> Result<String, DesktopCommandError> {
    Ok(std::env::current_dir()?.display().to_string())
}
#[tauri::command]
async fn list_sessions(
    cwd: Option<String>,
) -> Result<Vec<DesktopSessionInfo>, DesktopCommandError> {
    let cwd = match cwd {
        Some(cwd) => PathBuf::from(cwd),
        None => std::env::current_dir()?,
    };
    Ok(DesktopAgent::list_sessions(&cwd).await?)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            health,
            current_workspace,
            list_sessions
        ])
        .run(tauri::generate_context!())
        .expect("error while running Sabi Agent desktop application");
}
