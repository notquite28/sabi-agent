//! Slash command parsing and dispatch.
//!
//! Ported from:
//! - `pi/packages/coding-agent/src/core/slash-commands.ts`
//! - `pi/packages/coding-agent/src/modes/interactive/interactive-mode.ts`
//!
//! Simplifications:
//! - Starts with `/help`, `/quit`, `/clear`, `/session`, `/reload`, and `/skill:name`.
//! - No extension commands, prompt templates, settings UI, or model selector yet.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlashCommand {
    Help,
    Quit,
    Clear,
    Session,
    Reload,
    Skill { name: String, extra: String },
    Unknown(String),
}

pub fn parse(input: &str) -> Option<SlashCommand> {
    let input = input.trim();
    if !input.starts_with('/') {
        return None;
    }

    match input {
        "/help" => Some(SlashCommand::Help),
        "/quit" | "/exit" => Some(SlashCommand::Quit),
        "/clear" => Some(SlashCommand::Clear),
        "/session" => Some(SlashCommand::Session),
        "/reload" => Some(SlashCommand::Reload),
        _ if input.starts_with("/skill:") => {
            let rest = input.trim_start_matches("/skill:");
            let mut parts = rest.splitn(2, ' ');
            let name = parts.next().unwrap_or_default().to_string();
            let extra = parts.next().unwrap_or_default().trim().to_string();
            Some(SlashCommand::Skill { name, extra })
        }
        _ => Some(SlashCommand::Unknown(input.to_string())),
    }
}
