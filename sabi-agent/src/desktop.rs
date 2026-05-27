//! Desktop-facing API over the reusable Sabi Agent engine.
//!
//! Ported from:
//! - `pi/packages/agent/src/harness/session/session.ts`
//! - `pi/packages/coding-agent/src/modes/interactive/interactive-mode.ts`
//!
//! Simplifications:
//! - Exposes a small in-process API suitable for a future Tauri frontend.
//! - No RPC server, concurrent sessions, branch navigation, or streaming token deltas yet.

use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;

use crate::agent::run_agent_turn_with_events;
use crate::config::AppConfig;
use crate::events::AgentEvent;
use crate::llm::ModelConfig;
use crate::messages::Message;
use crate::session::{SessionStore, SessionSummary};
use crate::skills::{self, Skill};
use crate::system_prompt::{self, BuildOptions};
use crate::tools::builtin_tool_specs;

/// Runtime options needed by non-CLI frontends.
#[derive(Debug, Clone)]
pub struct DesktopOptions {
    pub cwd: PathBuf,
    pub model: ModelConfig,
}

impl DesktopOptions {
    /// Load options from the same config sources used by the CLI.
    pub fn load() -> Result<Self> {
        let config = AppConfig::load()?;
        Ok(Self::from_app_config(config))
    }

    pub fn from_app_config(config: AppConfig) -> Self {
        Self {
            cwd: config.cwd,
            model: ModelConfig {
                model: config.model,
                base_url: config.base_url,
                api_key: config.api_key,
            },
        }
    }
}

/// A compact session descriptor for session lists and headers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DesktopSessionInfo {
    pub id: String,
    pub path: PathBuf,
    pub cwd: PathBuf,
    pub message_count: usize,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub modified_at: OffsetDateTime,
}

/// Snapshot of the current in-memory frontend state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DesktopState {
    pub session: DesktopSessionInfo,
    pub skills: Vec<DesktopSkillInfo>,
    pub messages_len: usize,
}

/// Skill metadata safe to show in a UI list.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DesktopSkillInfo {
    pub name: String,
    pub description: String,
    pub file_path: PathBuf,
}

/// Stateful engine handle for desktop and other embedded frontends.
pub struct DesktopAgent {
    cwd: PathBuf,
    model: ModelConfig,
    system_prompt: String,
    skills: Vec<Skill>,
    messages: Vec<Message>,
    session: SessionStore,
    session_info: DesktopSessionInfo,
}

impl DesktopAgent {
    /// Start a new session for `options.cwd`.
    pub async fn start_new(options: DesktopOptions) -> Result<Self> {
        let skills = skills::discover(&options.cwd)?;
        let system_prompt = build_system_prompt(&options.cwd, &skills);
        let mut messages = Vec::new();
        inject_system_prompt(&mut messages, &system_prompt);
        let session = SessionStore::create(&options.cwd).await?;
        let session_info = session_info_from_store(&session, &options.cwd).await?;

        Ok(Self {
            cwd: options.cwd,
            model: options.model,
            system_prompt,
            skills,
            messages,
            session,
            session_info,
        })
    }

    /// Resume the latest non-empty session for `options.cwd`, or create a new one.
    pub async fn resume_latest(options: DesktopOptions) -> Result<Self> {
        let skills = skills::discover(&options.cwd)?;
        let system_prompt = build_system_prompt(&options.cwd, &skills);
        let (mut messages, session) = match SessionStore::latest(&options.cwd).await? {
            Some(session) => (session.load_messages().await?, session),
            None => (Vec::new(), SessionStore::create(&options.cwd).await?),
        };
        inject_system_prompt(&mut messages, &system_prompt);
        let session_info = session_info_from_store(&session, &options.cwd).await?;

        Ok(Self {
            cwd: options.cwd,
            model: options.model,
            system_prompt,
            skills,
            messages,
            session,
            session_info,
        })
    }

    /// Resume a specific session id for `options.cwd`.
    pub async fn resume_session(options: DesktopOptions, session_id: &str) -> Result<Option<Self>> {
        let Some(session) = SessionStore::open(&options.cwd, session_id).await? else {
            return Ok(None);
        };
        let skills = skills::discover(&options.cwd)?;
        let system_prompt = build_system_prompt(&options.cwd, &skills);
        let mut messages = session.load_messages().await?;
        inject_system_prompt(&mut messages, &system_prompt);
        let session_info = session_info_from_store(&session, &options.cwd).await?;

        Ok(Some(Self {
            cwd: options.cwd,
            model: options.model,
            system_prompt,
            skills,
            messages,
            session,
            session_info,
        }))
    }

    /// List non-empty sessions for `cwd`, newest modified first.
    pub async fn list_sessions(cwd: &Path) -> Result<Vec<DesktopSessionInfo>> {
        Ok(SessionStore::list(cwd)
            .await?
            .into_iter()
            .map(DesktopSessionInfo::from)
            .collect())
    }

    /// Send one user prompt through the agent loop.
    pub async fn send_prompt(
        &mut self,
        prompt: &str,
        emit: impl FnMut(AgentEvent),
        approve: impl FnMut(&str, &Value) -> bool,
    ) -> Result<String> {
        let reply = run_agent_turn_with_events(
            &self.model,
            &mut self.messages,
            &self.cwd,
            prompt,
            Some(&self.session),
            emit,
            approve,
        )
        .await?;
        self.session_info = session_info_from_store(&self.session, &self.cwd).await?;
        Ok(reply)
    }

    /// Clear only the in-memory transcript and keep the current session file.
    pub fn clear_conversation(&mut self) {
        self.messages.clear();
        inject_system_prompt(&mut self.messages, &self.system_prompt);
    }

    /// Create a fresh session file and clear the in-memory transcript.
    pub async fn start_new_session(&mut self) -> Result<()> {
        self.messages.clear();
        inject_system_prompt(&mut self.messages, &self.system_prompt);
        self.session = SessionStore::create(&self.cwd).await?;
        self.session_info = session_info_from_store(&self.session, &self.cwd).await?;
        Ok(())
    }

    /// Load the latest previous non-empty session, excluding the current file.
    pub async fn reload_previous_session(&mut self) -> Result<bool> {
        let Some(latest) =
            SessionStore::latest_excluding(&self.cwd, Some(&self.session.path)).await?
        else {
            return Ok(false);
        };
        self.messages = latest.load_messages().await?;
        inject_system_prompt(&mut self.messages, &self.system_prompt);
        self.session = latest;
        self.session_info = session_info_from_store(&self.session, &self.cwd).await?;
        self.refresh_skills()?;
        Ok(true)
    }

    /// Rediscover project and user skills and rebuild the system prompt.
    pub fn refresh_skills(&mut self) -> Result<()> {
        self.skills = skills::discover(&self.cwd)?;
        self.system_prompt = build_system_prompt(&self.cwd, &self.skills);
        inject_system_prompt(&mut self.messages, &self.system_prompt);
        Ok(())
    }

    pub fn state(&self) -> DesktopState {
        DesktopState {
            session: self.session_info(),
            skills: self.skill_info(),
            messages_len: self.messages.len(),
        }
    }

    pub fn session_info(&self) -> DesktopSessionInfo {
        let mut info = self.session_info.clone();
        info.message_count = self
            .messages
            .iter()
            .filter(|message| !matches!(message, Message::System { .. }))
            .count();
        info
    }

    pub fn skill_info(&self) -> Vec<DesktopSkillInfo> {
        self.skills
            .iter()
            .map(|skill| DesktopSkillInfo {
                name: skill.name.clone(),
                description: skill.description.clone(),
                file_path: skill.file_path.clone(),
            })
            .collect()
    }

    pub fn messages(&self) -> &[Message] {
        &self.messages
    }

    pub fn session(&self) -> &SessionStore {
        &self.session
    }
}

impl From<SessionSummary> for DesktopSessionInfo {
    fn from(summary: SessionSummary) -> Self {
        Self {
            id: summary.id,
            path: summary.path,
            cwd: PathBuf::from(summary.header.cwd),
            message_count: summary.message_count,
            created_at: summary.header.created_at,
            modified_at: summary.modified_at,
        }
    }
}

async fn session_info_from_store(session: &SessionStore, cwd: &Path) -> Result<DesktopSessionInfo> {
    let Some(summary) = session.summary().await? else {
        anyhow::bail!("session {} has no header", session.path.display());
    };
    let mut info = DesktopSessionInfo::from(summary);
    info.cwd = cwd.to_path_buf();
    Ok(info)
}

fn build_system_prompt(cwd: &Path, skills: &[Skill]) -> String {
    system_prompt::build(BuildOptions {
        cwd,
        tools: &builtin_tool_specs(),
        skills,
    })
}

fn inject_system_prompt(messages: &mut Vec<Message>, system_prompt: &str) {
    if matches!(messages.first(), Some(Message::System { .. })) {
        messages[0] = Message::system(system_prompt.to_string());
    } else {
        messages.insert(0, Message::system(system_prompt.to_string()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    fn test_options(cwd: PathBuf) -> DesktopOptions {
        DesktopOptions {
            cwd,
            model: ModelConfig {
                model: "test-model".to_string(),
                base_url: "http://localhost/v1".to_string(),
                api_key: "test-key".to_string(),
            },
        }
    }

    #[tokio::test]
    #[serial]
    async fn start_new_injects_system_prompt_and_reports_state() {
        let temp = tempfile::tempdir().unwrap();
        let old_home = std::env::var_os("HOME");
        std::env::set_var("HOME", temp.path());

        let agent = DesktopAgent::start_new(test_options(temp.path().to_path_buf()))
            .await
            .unwrap();
        let state = agent.state();

        assert!(matches!(
            agent.messages().first(),
            Some(Message::System { .. })
        ));
        assert_eq!(state.session.message_count, 0);
        assert_eq!(state.messages_len, 1);
        assert!(state.skills.iter().any(|skill| skill.name == "init"));
        assert!(state.skills.iter().any(|skill| skill.name == "review"));

        restore_home(old_home);
    }

    #[tokio::test]
    #[serial]
    async fn list_sessions_returns_non_empty_sessions_newest_first() {
        let temp = tempfile::tempdir().unwrap();
        let old_home = std::env::var_os("HOME");
        std::env::set_var("HOME", temp.path());
        let options = test_options(temp.path().to_path_buf());

        let empty = DesktopAgent::start_new(options.clone()).await.unwrap();
        let first = DesktopAgent::start_new(options.clone()).await.unwrap();
        first
            .session()
            .append_message(&Message::user("older"))
            .await
            .unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        let second = DesktopAgent::start_new(options.clone()).await.unwrap();
        second
            .session()
            .append_message(&Message::user("newer"))
            .await
            .unwrap();

        let sessions = DesktopAgent::list_sessions(&options.cwd).await.unwrap();

        assert_eq!(sessions.len(), 2);
        assert_eq!(sessions[0].id, second.session().id);
        assert_eq!(sessions[1].id, first.session().id);
        assert!(sessions
            .iter()
            .all(|session| session.id != empty.session().id));
        assert_eq!(sessions[0].message_count, 1);

        restore_home(old_home);
    }

    #[tokio::test]
    #[serial]
    async fn resume_session_loads_requested_session_by_id() {
        let temp = tempfile::tempdir().unwrap();
        let old_home = std::env::var_os("HOME");
        std::env::set_var("HOME", temp.path());
        let options = test_options(temp.path().to_path_buf());

        let agent = DesktopAgent::start_new(options.clone()).await.unwrap();
        agent
            .session()
            .append_message(&Message::user("specific"))
            .await
            .unwrap();

        let resumed = DesktopAgent::resume_session(options, &agent.session().id)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(resumed.session().id, agent.session().id);
        assert!(resumed
            .messages()
            .iter()
            .any(|message| matches!(message, Message::User { content } if content == "specific")));

        restore_home(old_home);
    }

    #[tokio::test]
    #[serial]
    async fn reload_previous_session_excludes_current_session() {
        let temp = tempfile::tempdir().unwrap();
        let old_home = std::env::var_os("HOME");
        std::env::set_var("HOME", temp.path());
        let options = test_options(temp.path().to_path_buf());

        let first = DesktopAgent::start_new(options.clone()).await.unwrap();
        first
            .session()
            .append_message(&Message::user("previous"))
            .await
            .unwrap();
        let mut second = DesktopAgent::start_new(options).await.unwrap();

        assert!(second.reload_previous_session().await.unwrap());
        assert!(second
            .messages()
            .iter()
            .any(|message| matches!(message, Message::User { content } if content == "previous")));

        restore_home(old_home);
    }

    fn restore_home(old_home: Option<std::ffi::OsString>) {
        if let Some(old_home) = old_home {
            std::env::set_var("HOME", old_home);
        } else {
            std::env::remove_var("HOME");
        }
    }
}
