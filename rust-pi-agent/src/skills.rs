//! Skill discovery and invocation formatting.
//!
//! Ported from:
//! - `pi/packages/coding-agent/src/core/skills.ts`
//! - `pi/packages/agent/src/harness/skills.ts`
//! - `pi/packages/agent/src/harness/system-prompt.ts`
//!
//! Simplifications:
//! - Starts with local `SKILL.md` discovery and minimal frontmatter fields.
//! - No Pi package integration or extension resource loader yet.

#[derive(Debug, Clone)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub file_path: std::path::PathBuf,
}
