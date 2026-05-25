//! CLI entry point for the Rust Pi agent.
//!
//! Ported from:
//! - `pi/packages/coding-agent/src/cli.ts`
//! - `pi/packages/coding-agent/src/main.ts`
//!
//! Simplifications:
//! - Starts with a single interactive mode.
//! - Does not include Pi's package commands, RPC mode, JSON mode, or update flows.

#![allow(dead_code)]

mod agent;
mod app;
mod config;
mod diff;
mod llm;
mod messages;
mod session;
mod skills;
mod slash;
mod tools;

use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "rust-pi-agent")]
#[command(about = "A small Rust learning port of Pi's core agent harness")]
struct Cli {
    /// Check provider connectivity, selected model availability, and chat support.
    #[arg(long)]
    check_provider: bool,

    /// Optional one-shot prompt. If omitted, interactive mode starts.
    prompt: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    app::run(cli.prompt, cli.check_provider).await
}
