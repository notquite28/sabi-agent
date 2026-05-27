//! JSONL session persistence.
//!
//! Ported from:
//! - `pi/packages/agent/src/harness/session/session.ts`
//! - `pi/packages/agent/src/harness/session/jsonl-storage.ts`
//!
//! Simplifications:
//! - Starts with a linear transcript.
//! - No tree, labels, fork, clone, branch summaries, or compaction entries yet.

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::messages::Message;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionHeader {
    pub kind: String,
    pub version: u32,
    pub id: String,
    pub cwd: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub struct SessionSummary {
    pub id: String,
    pub path: std::path::PathBuf,
    pub header: SessionHeader,
    pub message_count: usize,
    pub modified_at: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub struct SessionStore {
    pub id: String,
    pub path: std::path::PathBuf,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum SessionEntry<'a> {
    #[serde(rename = "header")]
    Header { header: &'a SessionHeader },
    #[serde(rename = "message")]
    Message {
        session_id: &'a str,
        #[serde(with = "time::serde::rfc3339")]
        timestamp: OffsetDateTime,
        message: &'a Message,
    },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum StoredSessionEntry {
    #[serde(rename = "header")]
    Header {
        #[serde(rename = "header")]
        header: SessionHeader,
    },
    #[serde(rename = "message")]
    Message { message: Message },
}

impl SessionStore {
    pub async fn create(cwd: &std::path::Path) -> anyhow::Result<Self> {
        let id = Uuid::now_v7().to_string();
        let dir = session_dir(cwd)?;
        tokio::fs::create_dir_all(&dir).await?;

        let path = dir.join(format!("{id}.jsonl"));
        let store = Self {
            id: id.clone(),
            path,
        };
        let header = SessionHeader {
            kind: "sabi-agent-session".to_string(),
            version: 1,
            id,
            cwd: cwd.display().to_string(),
            created_at: OffsetDateTime::now_utc(),
        };
        store
            .append_entry(&SessionEntry::Header { header: &header })
            .await?;
        Ok(store)
    }

    pub async fn open(cwd: &std::path::Path, id: &str) -> anyhow::Result<Option<Self>> {
        if Uuid::parse_str(id).is_err() {
            return Ok(None);
        }
        let path = session_dir(cwd)?.join(format!("{id}.jsonl"));
        match tokio::fs::metadata(&path).await {
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error.into()),
        }
        if !matches_cwd(&path, cwd).await.unwrap_or(false) {
            return Ok(None);
        }
        Ok(Some(Self {
            id: id.to_string(),
            path,
        }))
    }

    pub async fn list(cwd: &std::path::Path) -> anyhow::Result<Vec<SessionSummary>> {
        let dir = session_dir(cwd)?;
        let mut entries = match tokio::fs::read_dir(&dir).await {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(error.into()),
        };

        let mut sessions = Vec::new();
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|extension| extension.to_str()) != Some("jsonl") {
                continue;
            }
            let Some(summary) = summarize_session(&path).await.unwrap_or(None) else {
                continue;
            };
            if summary.header.cwd != cwd.display().to_string() || summary.message_count == 0 {
                continue;
            }
            sessions.push(summary);
        }

        sessions.sort_by(|left, right| right.modified_at.cmp(&left.modified_at));
        Ok(sessions)
    }

    pub async fn summary(&self) -> anyhow::Result<Option<SessionSummary>> {
        summarize_session(&self.path).await
    }

    pub async fn latest(cwd: &std::path::Path) -> anyhow::Result<Option<Self>> {
        Self::latest_excluding(cwd, None).await
    }

    pub async fn latest_excluding(
        cwd: &std::path::Path,
        excluded_path: Option<&std::path::Path>,
    ) -> anyhow::Result<Option<Self>> {
        let dir = session_dir(cwd)?;
        let mut entries = match tokio::fs::read_dir(&dir).await {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error.into()),
        };

        let mut latest: Option<(std::time::SystemTime, std::path::PathBuf)> = None;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|extension| extension.to_str()) != Some("jsonl") {
                continue;
            }
            if excluded_path.is_some_and(|excluded_path| path == excluded_path) {
                continue;
            }
            if !matches_cwd(&path, cwd).await.unwrap_or(false) {
                continue;
            }
            if !has_message_entries(&path).await.unwrap_or(false) {
                continue;
            }
            let modified = entry.metadata().await?.modified()?;
            match &latest {
                Some((latest_modified, _)) if modified <= *latest_modified => {}
                _ => latest = Some((modified, path)),
            }
        }

        let Some((_, path)) = latest else {
            return Ok(None);
        };
        let id = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("unknown-session")
            .to_string();
        Ok(Some(Self { id, path }))
    }

    pub async fn load_messages(&self) -> anyhow::Result<Vec<Message>> {
        let content = tokio::fs::read_to_string(&self.path).await?;
        let mut messages = Vec::new();
        for (index, line) in content.lines().enumerate() {
            let entry: StoredSessionEntry = serde_json::from_str(line).map_err(|error| {
                anyhow::anyhow!("invalid JSONL entry on line {}: {error}", index + 1)
            })?;
            match entry {
                StoredSessionEntry::Header { .. } => {}
                StoredSessionEntry::Message { message } => messages.push(message),
            }
        }
        Ok(messages)
    }

    pub async fn append_message(&self, message: &Message) -> anyhow::Result<()> {
        self.append_entry(&SessionEntry::Message {
            session_id: &self.id,
            timestamp: OffsetDateTime::now_utc(),
            message,
        })
        .await?;
        Ok(())
    }

    async fn append_entry(&self, entry: &SessionEntry<'_>) -> anyhow::Result<()> {
        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .await?;
        let line = serde_json::to_string(entry)?;
        file.write_all(line.as_bytes()).await?;
        file.write_all(b"\n").await?;
        Ok(())
    }
}

async fn matches_cwd(path: &std::path::Path, cwd: &std::path::Path) -> anyhow::Result<bool> {
    let content = tokio::fs::read_to_string(path).await?;
    let expected = cwd.display().to_string();
    for line in content.lines() {
        let entry: StoredSessionEntry = serde_json::from_str(line)?;
        if let StoredSessionEntry::Header { header } = entry {
            return Ok(header.cwd == expected);
        }
    }
    Ok(false)
}

async fn has_message_entries(path: &std::path::Path) -> anyhow::Result<bool> {
    let content = tokio::fs::read_to_string(path).await?;
    for line in content.lines() {
        let entry: StoredSessionEntry = serde_json::from_str(line)?;
        if matches!(entry, StoredSessionEntry::Message { .. }) {
            return Ok(true);
        }
    }
    Ok(false)
}

async fn summarize_session(path: &std::path::Path) -> anyhow::Result<Option<SessionSummary>> {
    let content = tokio::fs::read_to_string(path).await?;
    let mut header = None;
    let mut message_count = 0;
    for line in content.lines() {
        let entry: StoredSessionEntry = serde_json::from_str(line)?;
        match entry {
            StoredSessionEntry::Header { header: found } => header = Some(found),
            StoredSessionEntry::Message { .. } => message_count += 1,
        }
    }

    let Some(header) = header else {
        return Ok(None);
    };
    let modified_at = tokio::fs::metadata(path)
        .await?
        .modified()
        .map(OffsetDateTime::from)?;
    let id = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("unknown-session")
        .to_string();

    Ok(Some(SessionSummary {
        id,
        path: path.to_path_buf(),
        header,
        message_count,
        modified_at,
    }))
}

fn session_dir(cwd: &std::path::Path) -> anyhow::Result<std::path::PathBuf> {
    let base = crate::config::sessions_dir()
        .ok_or_else(|| anyhow::anyhow!("failed to resolve session directory"))?;
    let workspace_name = cwd
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("workspace");
    Ok(base.join(workspace_name))
}
