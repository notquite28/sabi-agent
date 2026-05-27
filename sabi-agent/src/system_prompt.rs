//! System prompt construction.
//!
//! Ported from:
//! - `pi/packages/coding-agent/src/core/system-prompt.ts`
//!
//! Simplifications:
//! - No custom prompt override or append-system-prompt flags yet.
//! - Loads AGENTS.md from cwd if present.
//! - Includes skill summaries when skills are available.

use std::path::Path;

use anyhow::{Context, Result};

use crate::skills::Skill;
use crate::tools::ToolSpec;

pub struct BuildOptions<'a> {
    pub cwd: &'a Path,
    pub tools: &'a [ToolSpec],
    pub skills: &'a [Skill],
}

pub fn build(options: BuildOptions<'_>) -> String {
    let BuildOptions { cwd, tools, skills } = options;

    let now = time::OffsetDateTime::now_utc();
    let date = format!(
        "{:04}-{:02}-{:02}",
        now.year(),
        now.month() as u8,
        now.day()
    );
    let prompt_cwd = cwd.display().to_string().replace('\\', "/");

    // Build tools list
    let tools_list = if tools.is_empty() {
        "(none)".to_string()
    } else {
        tools
            .iter()
            .map(|tool| format!("- {}: {}", tool.name, tool.description))
            .collect::<Vec<_>>()
            .join("\n")
    };

    // Build guidelines based on available tools
    let mut guidelines = Vec::new();
    let tool_names: Vec<&str> = tools.iter().map(|t| t.name).collect();
    let has_bash = tool_names.contains(&"bash");
    let has_grep = tool_names.contains(&"grep");
    let has_find = tool_names.contains(&"find");
    let has_ls = tool_names.contains(&"ls");

    if has_bash && !has_grep && !has_find && !has_ls {
        guidelines.push("Use bash for file operations like ls, rg, find");
    } else if has_bash && (has_grep || has_find || has_ls) {
        guidelines.push(
            "Prefer grep/find/ls tools over bash for file exploration (faster, respects .gitignore)",
        );
    }

    guidelines.push("Be concise in your responses");
    guidelines.push("Show file paths clearly when working with files");

    let guidelines_text = guidelines
        .iter()
        .map(|g| format!("- {g}"))
        .collect::<Vec<_>>()
        .join("\n");

    // Load project context from AGENTS.md
    let mut context_section = String::new();
    if let Ok(agents_md) = load_agents_md(cwd) {
        context_section.push_str("\n\n<project_context>\n\n");
        context_section.push_str("Project-specific instructions and guidelines:\n\n");
        context_section.push_str(&format!(
            "<project_instructions path=\"AGENTS.md\">\n{}\n</project_instructions>\n",
            agents_md
        ));
        context_section.push_str("</project_context>");
    }

    // Skills section
    let mut skills_section = String::new();
    if !skills.is_empty() {
        let available_skills: Vec<&Skill> = skills
            .iter()
            .filter(|s| !s.disable_model_invocation)
            .collect();
        if !available_skills.is_empty() {
            skills_section.push_str("\n\nAvailable skills. Invoke one by asking the user to run `/skill:name` when it is relevant:\n");
            for skill in available_skills {
                skills_section.push_str(&format!("- {}: {}\n", skill.name, skill.description));
            }
        }
    }

    format!(
        "You are Sabi, an expert coding assistant. You help users by reading files, executing commands, editing code, and writing new files.\n\nAvailable tools:\n{tools_list}\n\nGuidelines:\n{guidelines_text}{context_section}{skills_section}\n\nCurrent date: {date}\nCurrent working directory: {prompt_cwd}"
    )
}

fn load_agents_md(cwd: &Path) -> Result<String> {
    let path = cwd.join("AGENTS.md");
    if !path.exists() {
        anyhow::bail!("AGENTS.md not found");
    }
    std::fs::read_to_string(&path)
        .map(|content| content.trim().to_string())
        .with_context(|| format!("failed to read {}", path.display()))
}
