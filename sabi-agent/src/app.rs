//! Top-level application flow.
//!
//! Ported from:
//! - `pi/packages/coding-agent/src/modes/interactive/interactive-mode.ts`
//! - `pi/packages/coding-agent/src/modes/print-mode.ts`
//!
//! Simplifications:
//! - Uses a plain readline-style prompt instead of Pi's custom TUI.
//! - Uses non-streaming provider responses before adding tool calls and TUI rendering.

use anyhow::Result;
use rustyline::DefaultEditor;
use serde_json::Value;

use crate::agent::run_agent_turn_with_events;
use crate::config::AppConfig;
use crate::events::AgentEvent;
use crate::llm::{check_provider, ModelConfig};
use crate::messages::Message;
use crate::session::SessionStore;
use crate::skills::{self, Skill};
use crate::slash::{self, SlashCommand};

pub async fn run(
    prompt_words: Vec<String>,
    should_check_provider: bool,
    should_resume: bool,
) -> Result<()> {
    let config = AppConfig::load()?;
    let cwd = config.cwd.clone();
    let model = ModelConfig {
        model: config.model,
        base_url: config.base_url,
        api_key: config.api_key,
    };

    if should_check_provider {
        check_provider(&model).await?;
        return Ok(());
    }

    let mut skills = skills::discover(&cwd)?;

    let (mut messages, mut session, resumed) = if should_resume {
        match SessionStore::latest(&cwd).await? {
            Some(session) => {
                let messages = session.load_messages().await?;
                (messages, session, true)
            }
            None => (Vec::new(), SessionStore::create(&cwd).await?, false),
        }
    } else {
        (Vec::new(), SessionStore::create(&cwd).await?, false)
    };

    if !prompt_words.is_empty() {
        let prompt = prompt_words.join(" ");
        let prompt = prompt_with_skills(&prompt, &skills);
        run_agent_turn_with_events(
            &model,
            &mut messages,
            &cwd,
            &prompt,
            Some(&session),
            render_event,
            |_, _| true,
        )
        .await?;
        return Ok(());
    }

    println!(
        "sabi-agent ready with read/write/edit/bash/ls/grep/find tools. Type /help for commands or /quit to exit."
    );
    println!("session: {}", session.path.display());
    if resumed {
        println!("resumed messages: {}", messages.len());
    }
    let mut fiwb_mode = false;

    let mut editor = DefaultEditor::new()?;
    loop {
        let line = editor.readline("> ")?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let _ = editor.add_history_entry(trimmed);
        if let Some(command) = slash::parse(trimmed) {
            if handle_command(
                command,
                &mut messages,
                &mut session,
                &cwd,
                &mut fiwb_mode,
                &mut skills,
                &model,
            )
            .await?
            {
                break;
            }
            continue;
        }

        run_agent_turn_with_events(
            &model,
            &mut messages,
            &cwd,
            &prompt_with_skills(trimmed, &skills),
            Some(&session),
            render_event,
            |name, args| approve_tool(name, args, fiwb_mode),
        )
        .await?;
    }

    Ok(())
}

fn prompt_with_skills(prompt: &str, skills: &[Skill]) -> String {
    let available = skills::format_available(skills);
    if available.is_empty() {
        return prompt.to_string();
    }
    format!("{available}\nUser request:\n{prompt}")
}

fn render_event(event: AgentEvent) {
    match event {
        AgentEvent::AssistantText { text } => println!("\n{text}\n"),
        AgentEvent::ToolStarted { name, args, .. } => println!("tool: {name} {args}"),
        AgentEvent::ToolFinished {
            output, is_error, ..
        } => {
            if is_error {
                println!("tool error: {output}");
            } else {
                println!("tool result:\n{}", preview_tool_output(&output));
            }
        }
        AgentEvent::DiffReady { path, rendered, .. } => println!("diff: {path}\n{rendered}"),
        AgentEvent::FileChanged { path } => println!("file changed: {path}"),
        AgentEvent::Error { message } => println!("error: {message}"),
    }
}

fn preview_tool_output(output: &str) -> String {
    const MAX_LINES: usize = 80;

    let lines: Vec<&str> = output.lines().collect();
    if lines.len() <= MAX_LINES {
        return output.to_string();
    }

    let mut preview = lines[..MAX_LINES].join("\n");
    preview.push_str(&format!(
        "\n[tool output preview truncated: showing {MAX_LINES} of {} lines]",
        lines.len()
    ));
    preview
}

async fn handle_command(
    command: SlashCommand,
    messages: &mut Vec<Message>,
    session: &mut SessionStore,
    cwd: &std::path::Path,
    fiwb_mode: &mut bool,
    skills: &mut Vec<Skill>,
    model: &ModelConfig,
) -> Result<bool> {
    match command {
        SlashCommand::Help => {
            println!("commands: /help, /clear, /new, /session, /reload, /skill:name, /fiwb, /yolo, /quit");
            Ok(false)
        }
        SlashCommand::Quit => Ok(true),
        SlashCommand::Clear => {
            messages.clear();
            println!("conversation cleared");
            Ok(false)
        }
        SlashCommand::New => {
            messages.clear();
            *session = SessionStore::create(cwd).await?;
            println!("new session: {}", session.path.display());
            Ok(false)
        }
        SlashCommand::Session => {
            println!("messages in memory: {}", messages.len());
            println!("session id: {}", session.id);
            println!("session file: {}", session.path.display());
            println!("skills loaded: {}", skills.len());
            println!("fiwb mode: {}", if *fiwb_mode { "on" } else { "off" });
            Ok(false)
        }
        SlashCommand::Fiwb => {
            *fiwb_mode = !*fiwb_mode;
            if *fiwb_mode {
                println!("FIWB mode enabled for this process only. Risky tools will run without approval until restart or /fiwb.");
            } else {
                println!("FIWB mode disabled. Risky tools require approval again.");
            }
            Ok(false)
        }
        SlashCommand::Reload => {
            let Some(latest) = SessionStore::latest_excluding(cwd, Some(&session.path)).await?
            else {
                println!("no previous session found");
                return Ok(false);
            };
            let loaded = latest.load_messages().await?;
            let count = loaded.len();
            let path = latest.path.display().to_string();
            *messages = loaded;
            *session = latest;
            *skills = skills::discover(cwd)?;
            println!("loaded {count} messages from {path}");
            println!("reloaded {} skills", skills.len());
            Ok(false)
        }
        SlashCommand::Skill { name, extra } => {
            let Some(skill) = skills.iter().find(|skill| skill.name == name) else {
                println!("unknown skill: {name}");
                if !skills.is_empty() {
                    println!(
                        "available skills: {}",
                        skills
                            .iter()
                            .map(|skill| skill.name.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                }
                return Ok(false);
            };
            if skill.disable_model_invocation {
                println!("skill cannot be invoked by the model: {name}");
                return Ok(false);
            }
            let prompt = skills::format_invocation(skill, &extra);
            run_agent_turn_with_events(
                model,
                messages,
                cwd,
                &prompt,
                Some(session),
                render_event,
                |tool_name, args| approve_tool(tool_name, args, *fiwb_mode),
            )
            .await?;
            Ok(false)
        }
        SlashCommand::Unknown(name) => {
            println!("unknown command: {name}");
            Ok(false)
        }
    }
}

fn approve_tool(name: &str, args: &Value, fiwb_mode: bool) -> bool {
    if !is_risky_tool(name) || fiwb_mode {
        return true;
    }

    println!(
        "approval required: {} {}",
        name,
        approval_summary(name, args)
    );
    println!("Approve? [y/N]");

    let mut answer = String::new();
    if std::io::stdin().read_line(&mut answer).is_err() {
        return false;
    }
    matches!(answer.trim(), "y" | "Y" | "yes" | "YES")
}

fn is_risky_tool(name: &str) -> bool {
    matches!(name, "write" | "edit" | "bash")
}

fn approval_summary(name: &str, args: &Value) -> String {
    match name {
        "bash" => args
            .get("command")
            .and_then(Value::as_str)
            .map(|command| format!("command={command:?}"))
            .unwrap_or_else(|| args.to_string()),
        "write" | "edit" => args
            .get("path")
            .and_then(Value::as_str)
            .map(|path| format!("path={path:?}"))
            .unwrap_or_else(|| args.to_string()),
        _ => args.to_string(),
    }
}
