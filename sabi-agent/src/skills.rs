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

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use ignore::WalkBuilder;

#[derive(Debug, Clone)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub file_path: std::path::PathBuf,
    pub content: String,
    pub disable_model_invocation: bool,
}

pub fn discover(cwd: &Path) -> Result<Vec<Skill>> {
    let mut skills = vec![builtin_init_skill()];
    for root in skill_roots(cwd) {
        if !root.exists() {
            continue;
        }
        for entry in WalkBuilder::new(&root).build() {
            let entry = entry.with_context(|| format!("failed to walk {}", root.display()))?;
            if !entry
                .file_type()
                .is_some_and(|file_type| file_type.is_file())
            {
                continue;
            }
            if entry.file_name() != "SKILL.md" {
                continue;
            }
            match parse_skill(entry.path()) {
                Ok(skill) => skills.push(skill),
                Err(error) => eprintln!("skipping skill {}: {error}", entry.path().display()),
            }
        }
    }

    skills.sort_by(|left, right| left.name.cmp(&right.name));
    skills.dedup_by(|left, right| left.name == right.name);
    Ok(skills)
}

fn builtin_init_skill() -> Skill {
    Skill {
        name: "init".to_string(),
        description: "Create or update a compact repository AGENTS.md instruction file."
            .to_string(),
        file_path: PathBuf::from("<built-in>/init"),
        content: INIT_SKILL.trim().to_string(),
        disable_model_invocation: false,
    }
}

const INIT_SKILL: &str = r#"Create or update `AGENTS.md` for this repository.

The goal is a compact instruction file that helps future OpenCode sessions avoid mistakes and ramp up quickly. Every line should answer: "Would an agent likely miss this without help?" If not, leave it out.

User-provided focus or constraints (honor these):
$ARGUMENTS

## How to investigate

Read the highest-value sources first:
- `README*`, root manifests, workspace config, lockfiles
- build, test, lint, formatter, typecheck, and codegen config
- CI workflows and pre-commit / task runner config
- existing instruction files (`AGENTS.md`, `CLAUDE.md`, `.cursor/rules/`, `.cursorrules`, `.github/copilot-instructions.md`)
- repo-local OpenCode config such as `opencode.json`

If architecture is still unclear after reading config and docs, inspect a small number of representative code files to find the real entrypoints, package boundaries, and execution flow. Prefer reading the files that explain how the system is wired together over random leaf files.

Prefer executable sources of truth over prose. If docs conflict with config or scripts, trust the executable source and only keep what you can verify.

## What to extract

Look for the highest-signal facts for an agent working in this repo:
- exact developer commands, especially non-obvious ones
- how to run a single test, a single package, or a focused verification step
- required command order when it matters, such as `lint -> typecheck -> test`
- monorepo or multi-package boundaries, ownership of major directories, and the real app/library entrypoints
- framework or toolchain quirks: generated code, migrations, codegen, build artifacts, special env loading, dev servers, infra deploy flow
- repo-specific style or workflow conventions that differ from defaults
- testing quirks: fixtures, integration test prerequisites, snapshot workflows, required services, flaky or expensive suites
- important constraints from existing instruction files worth preserving

Good `AGENTS.md` content is usually hard-earned context that took reading multiple files to infer.

## Questions

Only ask the user questions if the repo cannot answer something important. Use the `question` tool for one short batch at most.

Good questions:
- undocumented team conventions
- branch / PR / release expectations
- missing setup or test prerequisites that are known but not written down

Do not ask about anything the repo already makes clear.

## Writing rules

Include only high-signal, repo-specific guidance such as:
- exact commands and shortcuts the agent would otherwise guess wrong
- architecture notes that are not obvious from filenames
- conventions that differ from language or framework defaults
- setup requirements, environment quirks, and operational gotchas
- references to existing instruction sources that matter

Exclude:
- generic software advice
- long tutorials or exhaustive file trees
- obvious language conventions
- speculative claims or anything you could not verify
- content better stored in another file referenced via `opencode.json` `instructions`

When in doubt, omit.

Prefer short sections and bullets. If the repo is simple, keep the file simple. If the repo is large, summarize the few structural facts that actually change how an agent should work.

If `AGENTS.md` already exists at the target path, improve it in place rather than rewriting blindly. Preserve verified useful guidance, delete fluff or stale claims, and reconcile it with the current codebase."#;

pub fn format_invocation(skill: &Skill, extra: &str) -> String {
    let mut prompt = format!(
        "Use the following skill instructions.\n\n# Skill: {}\n\nDescription: {}\n\n{}",
        skill.name, skill.description, skill.content
    );
    if !extra.trim().is_empty() {
        prompt = prompt.replace("$ARGUMENTS", extra.trim());
        if !prompt.contains(extra.trim()) {
            prompt.push_str("\n\nAdditional user instructions:\n");
            prompt.push_str(extra.trim());
        }
    } else {
        prompt = prompt.replace("$ARGUMENTS", "");
    }
    prompt
}

pub fn format_available(skills: &[Skill]) -> String {
    let available: Vec<&Skill> = skills
        .iter()
        .filter(|skill| !skill.disable_model_invocation)
        .collect();
    if available.is_empty() {
        return String::new();
    }

    let mut text = String::from("Available skills. Invoke one by asking the user to run `/skill:name` when it is relevant:\n");
    for skill in available {
        text.push_str(&format!("- {}: {}\n", skill.name, skill.description));
    }
    text
}

fn skill_roots(cwd: &Path) -> Vec<PathBuf> {
    let mut roots = vec![cwd.join(".sabi/skills")];
    if let Some(home) = std::env::var_os("HOME") {
        let home = PathBuf::from(home);
        roots.push(home.join(".sabi/skills"));
    }
    roots
}

fn parse_skill(path: &Path) -> Result<Skill> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read skill {}", path.display()))?;
    let (frontmatter, body) =
        split_frontmatter(&content).ok_or_else(|| anyhow::anyhow!("missing YAML frontmatter"))?;
    let name =
        frontmatter_value(frontmatter, "name").ok_or_else(|| anyhow::anyhow!("missing name"))?;
    let description = frontmatter_value(frontmatter, "description")
        .ok_or_else(|| anyhow::anyhow!("missing description"))?;
    let disable_model_invocation = frontmatter_value(frontmatter, "disable-model-invocation")
        .is_some_and(|value| value == "true");

    Ok(Skill {
        name,
        description,
        file_path: path.to_path_buf(),
        content: body.trim().to_string(),
        disable_model_invocation,
    })
}

fn split_frontmatter(content: &str) -> Option<(&str, &str)> {
    let rest = content
        .strip_prefix("---\r\n")
        .or_else(|| content.strip_prefix("---\n"))?;
    let (frontmatter, body) = rest
        .split_once("\r\n---\r\n")
        .or_else(|| rest.split_once("\n---\n"))?;
    Some((frontmatter, body))
}

fn frontmatter_value(frontmatter: &str, key: &str) -> Option<String> {
    for line in frontmatter.lines() {
        let Some((line_key, value)) = line.split_once(':') else {
            continue;
        };
        if line_key.trim() == key {
            let value = value.trim();
            let value = value
                .strip_prefix('"')
                .and_then(|v| v.strip_suffix('"'))
                .unwrap_or(value);
            return Some(value.to_string());
        }
    }
    None
}
