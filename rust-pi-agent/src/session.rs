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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionHeader {
    pub kind: String,
    pub version: u32,
    pub id: String,
    pub cwd: String,
}
