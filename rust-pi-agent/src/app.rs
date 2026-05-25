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

use crate::agent::run_agent_turn;
use crate::config::AppConfig;
use crate::llm::{check_provider, ModelConfig};
use crate::messages::Message;
use crate::slash::{self, SlashCommand};

pub async fn run(prompt_words: Vec<String>, should_check_provider: bool) -> Result<()> {
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

    let mut messages = Vec::new();

    if !prompt_words.is_empty() {
        let prompt = prompt_words.join(" ");
        run_agent_turn(&model, &mut messages, &cwd, &prompt).await?;
        return Ok(());
    }

    println!(
        "rust-pi-agent ready with read/write/bash tools. Type /help for commands or /quit to exit."
    );

    let mut editor = DefaultEditor::new()?;
    loop {
        let line = editor.readline("> ")?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let _ = editor.add_history_entry(trimmed);
        if let Some(command) = slash::parse(trimmed) {
            if handle_command(command, &mut messages) {
                break;
            }
            continue;
        }

        run_agent_turn(&model, &mut messages, &cwd, trimmed).await?;
    }

    Ok(())
}

fn handle_command(command: SlashCommand, messages: &mut Vec<Message>) -> bool {
    match command {
        SlashCommand::Help => {
            println!("commands: /help, /clear, /session, /reload, /quit");
            false
        }
        SlashCommand::Quit => true,
        SlashCommand::Clear => {
            messages.clear();
            println!("conversation cleared");
            false
        }
        SlashCommand::Session => {
            println!("messages in memory: {}", messages.len());
            false
        }
        SlashCommand::Reload => {
            println!("reload is not implemented yet");
            false
        }
        SlashCommand::Skill { name, .. } => {
            println!("skill invocation is not implemented yet: {name}");
            false
        }
        SlashCommand::Unknown(name) => {
            println!("unknown command: {name}");
            false
        }
    }
}
