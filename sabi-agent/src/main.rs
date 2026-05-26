//! CLI entry point for Sabi Agent.
//!
//! Ported from:
//! - `pi/packages/coding-agent/src/cli.ts`
//! - `pi/packages/coding-agent/src/main.ts`
//!
//! Simplifications:
//! - Starts with a single interactive mode.
//! - Does not include Pi's package commands, RPC mode, JSON mode, or update flows.

use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "sabi-agent")]
#[command(about = "A small Rust coding agent harness")]
struct Cli {
    /// Check provider connectivity, selected model availability, and chat support.
    #[arg(long)]
    check_provider: bool,

    /// Resume the latest non-empty session for the current working directory.
    #[arg(long)]
    resume: bool,

    /// Optional one-shot prompt. If omitted, interactive mode starts.
    prompt: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    sabi_agent::app::run(cli.prompt, cli.check_provider, cli.resume).await
}
