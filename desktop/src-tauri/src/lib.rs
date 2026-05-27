//! Tauri command bridge for the Sabi Agent desktop frontend.
//!
//! Ported from:
//! - `sabi-agent/src/desktop.rs`
//!
//! Simplifications:
//! - Workspace selection is frontend state; commands accept explicit cwd strings.
//! - Prompt execution returns a batch of events; live event streaming and approval UI can come later.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::mpsc,
};

use anyhow::Context;
use sabi_agent::{
    approval::ToolApprovalRequest,
    config::AppConfig,
    desktop::{DesktopAgent, DesktopOptions, DesktopSessionInfo, DesktopSkillInfo},
    events::AgentEvent,
    messages::Message,
    session::SessionStore,
    skills,
};
use serde::Serialize;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::Mutex;

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

#[derive(Default)]
struct DesktopAppState {
    agent: Mutex<Option<DesktopAgent>>,
    approvals: std::sync::Mutex<HashMap<String, mpsc::Sender<bool>>>,
}

#[derive(Debug, Clone, Serialize)]
struct PromptResponse {
    reply: String,
    events: Vec<AgentEvent>,
    session: DesktopSessionInfo,
}

#[derive(Debug, Clone, Serialize)]
struct SessionTranscriptResponse {
    session: DesktopSessionInfo,
    messages: Vec<TranscriptMessage>,
}

#[derive(Debug, Clone, Serialize)]
struct TranscriptMessage {
    role: &'static str,
    content: String,
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

#[tauri::command]
async fn start_or_resume_session(
    state: State<'_, DesktopAppState>,
    cwd: Option<String>,
) -> Result<DesktopSessionInfo, DesktopCommandError> {
    let cwd = cwd_from_option(cwd)?;
    let options = desktop_options_for_cwd(cwd)?;
    let agent = DesktopAgent::resume_latest(options).await?;
    let session = agent.session_info();
    *state.agent.lock().await = Some(agent);
    Ok(session)
}

#[tauri::command]
async fn start_new_session(
    state: State<'_, DesktopAppState>,
    cwd: Option<String>,
) -> Result<DesktopSessionInfo, DesktopCommandError> {
    let cwd = cwd_from_option(cwd)?;
    let options = desktop_options_for_cwd(cwd)?;
    let agent = DesktopAgent::start_new(options).await?;
    let session = agent.session_info();
    *state.agent.lock().await = Some(agent);
    Ok(session)
}

#[tauri::command]
async fn resume_session(
    state: State<'_, DesktopAppState>,
    cwd: Option<String>,
    id: String,
) -> Result<Option<SessionTranscriptResponse>, DesktopCommandError> {
    let cwd = cwd_from_option(cwd)?;
    let options = desktop_options_for_cwd(cwd)?;
    let Some(agent) = DesktopAgent::resume_session(options, &id).await? else {
        return Ok(None);
    };
    let response = SessionTranscriptResponse {
        session: agent.session_info(),
        messages: transcript_messages(agent.messages()),
    };
    *state.agent.lock().await = Some(agent);
    Ok(Some(response))
}

#[tauri::command]
async fn send_prompt(
    app: AppHandle,
    state: State<'_, DesktopAppState>,
    cwd: Option<String>,
    prompt: String,
) -> Result<PromptResponse, DesktopCommandError> {
    let cwd = cwd_from_option(cwd)?;
    let mut guard = state.agent.lock().await;
    if guard
        .as_ref()
        .is_none_or(|agent| agent.session_info().cwd != cwd)
    {
        let options = desktop_options_for_cwd(cwd)?;
        *guard = Some(DesktopAgent::resume_latest(options).await?);
    }
    let agent = guard.as_mut().expect("agent initialized above");
    let mut events = Vec::new();
    let approvals = &state.approvals;
    let reply = agent
        .send_prompt(
            &prompt,
            |event| events.push(event),
            |approval| request_approval(&app, approvals, approval),
        )
        .await?;
    Ok(PromptResponse {
        reply,
        events,
        session: agent.session_info(),
    })
}

#[tauri::command]
fn answer_approval(
    state: State<'_, DesktopAppState>,
    id: String,
    approved: bool,
) -> Result<(), DesktopCommandError> {
    if let Some(sender) = state
        .approvals
        .lock()
        .expect("approval lock poisoned")
        .remove(&id)
    {
        let _ = sender.send(approved);
    }
    Ok(())
}

fn cwd_from_option(cwd: Option<String>) -> Result<PathBuf, std::io::Error> {
    match cwd {
        Some(cwd) => Ok(PathBuf::from(cwd)),
        None => std::env::current_dir(),
    }
}

fn desktop_options_for_cwd(cwd: PathBuf) -> anyhow::Result<DesktopOptions> {
    Ok(DesktopOptions::from_app_config(AppConfig::load_for_cwd(
        cwd,
    )?))
}

fn request_approval(
    app: &AppHandle,
    approvals: &std::sync::Mutex<HashMap<String, mpsc::Sender<bool>>>,
    approval: &ToolApprovalRequest,
) -> bool {
    let (sender, receiver) = mpsc::channel();
    approvals
        .lock()
        .expect("approval lock poisoned")
        .insert(approval.id.clone(), sender);
    if app.emit("tool-approval-requested", approval).is_err() {
        approvals
            .lock()
            .expect("approval lock poisoned")
            .remove(&approval.id);
        return false;
    }
    receiver.recv().unwrap_or(false)
}

fn transcript_messages(messages: &[Message]) -> Vec<TranscriptMessage> {
    messages
        .iter()
        .filter_map(|message| match message {
            Message::System { .. } => None,
            Message::User { content } => Some(TranscriptMessage {
                role: "user",
                content: content.clone(),
            }),
            Message::Assistant { content, .. } if content.trim().is_empty() => None,
            Message::Assistant { content, .. } => Some(TranscriptMessage {
                role: "assistant",
                content: content.clone(),
            }),
            Message::ToolResult {
                tool_name, content, ..
            } => Some(TranscriptMessage {
                role: "tool",
                content: format!("{tool_name}: {content}"),
            }),
        })
        .collect()
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
        .manage(DesktopAppState::default())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            health,
            current_workspace,
            list_sessions,
            list_skills,
            list_workspace_files,
            delete_session,
            start_or_resume_session,
            start_new_session,
            resume_session,
            send_prompt,
            answer_approval
        ])
        .run(tauri::generate_context!())
        .expect("error while running Sabi Agent desktop application");
}
