//! Tauri command bridge for the Sabi Agent desktop frontend.
//!
//! Ported from:
//! - `sabi-agent/src/desktop.rs`
//!
//! Simplifications:
//! - Workspace selection is frontend state; commands accept explicit cwd strings.
//! - Prompt execution, event streaming, and approval responses are added after the shell is stable.

use std::path::{Path, PathBuf};

use anyhow::Context;
use sabi_agent::{
    desktop::{DesktopAgent, DesktopSessionInfo, DesktopSkillInfo},
    session::SessionStore,
    skills,
};
use serde::Serialize;

const MAX_FILE_SUGGESTIONS: usize = 80;
const MAX_WALK_ENTRIES: usize = 4_000;

#[derive(Debug, thiserror::Error)]
enum DesktopCommandError {
    #[error("failed to resolve current directory: {0}")]
    CurrentDir(#[from] std::io::Error),
    #[error("agent error: {0}")]
    Agent(#[from] anyhow::Error),
}

#[derive(Debug, Clone, Serialize)]
struct DesktopFileSuggestion {
    path: String,
    name: String,
    is_dir: bool,
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
    let cwd = cwd_from_option(cwd)?;
    Ok(DesktopAgent::list_sessions(&cwd).await?)
}

#[tauri::command]
fn list_skills(cwd: Option<String>) -> Result<Vec<DesktopSkillInfo>, DesktopCommandError> {
    let cwd = cwd_from_option(cwd)?;
    let skills = skills::discover(&cwd)?;
    Ok(skills
        .into_iter()
        .map(|skill| DesktopSkillInfo {
            name: skill.name,
            description: skill.description,
            file_path: skill.file_path,
        })
        .collect())
}

#[tauri::command]
fn list_workspace_files(
    cwd: Option<String>,
    query: Option<String>,
) -> Result<Vec<DesktopFileSuggestion>, DesktopCommandError> {
    let cwd = cwd_from_option(cwd)?;
    let query = query.unwrap_or_default().to_lowercase();
    Ok(workspace_file_suggestions(&cwd, &query)?)
}

#[tauri::command]
async fn delete_session(cwd: Option<String>, id: String) -> Result<bool, DesktopCommandError> {
    let cwd = cwd_from_option(cwd)?;
    Ok(SessionStore::delete(&cwd, &id).await?)
}

fn cwd_from_option(cwd: Option<String>) -> Result<PathBuf, std::io::Error> {
    match cwd {
        Some(cwd) => Ok(PathBuf::from(cwd)),
        None => std::env::current_dir(),
    }
}

fn workspace_file_suggestions(
    cwd: &Path,
    query: &str,
) -> anyhow::Result<Vec<DesktopFileSuggestion>> {
    let mut suggestions = Vec::new();
    let mut stack = vec![cwd.to_path_buf()];
    let mut visited = 0usize;

    while let Some(dir) = stack.pop() {
        visited += 1;
        if visited > MAX_WALK_ENTRIES || suggestions.len() >= MAX_FILE_SUGGESTIONS {
            break;
        }

        let entries = match std::fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(_) => continue,
        };

        for entry in entries {
            let entry = entry.with_context(|| format!("failed to read {}", dir.display()))?;
            let path = entry.path();
            let file_name = entry.file_name();
            let file_name = file_name.to_string_lossy();

            if should_skip_entry(&file_name) {
                continue;
            }

            let metadata = match entry.metadata() {
                Ok(metadata) => metadata,
                Err(_) => continue,
            };
            let is_dir = metadata.is_dir();
            let relative = path.strip_prefix(cwd).unwrap_or(&path);
            let relative = relative.to_string_lossy().replace('\\', "/");

            if query.is_empty() || relative.to_lowercase().contains(query) {
                suggestions.push(DesktopFileSuggestion {
                    name: file_name.into_owned(),
                    path: relative,
                    is_dir,
                });
                if suggestions.len() >= MAX_FILE_SUGGESTIONS {
                    break;
                }
            }

            if is_dir {
                stack.push(path);
            }
        }
    }

    suggestions.sort_by(|left, right| {
        left.is_dir
            .cmp(&right.is_dir)
            .reverse()
            .then_with(|| left.path.cmp(&right.path))
    });
    Ok(suggestions)
}

fn should_skip_entry(file_name: &str) -> bool {
    matches!(
        file_name,
        ".git" | "node_modules" | "target" | "dist" | ".next" | ".cache"
    ) || file_name.starts_with('.')
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            health,
            current_workspace,
            list_sessions,
            list_skills,
            list_workspace_files,
            delete_session
        ])
        .run(tauri::generate_context!())
        .expect("error while running Sabi Agent desktop application");
}
