# Repository Instructions

## Scope

- `pi/` is the original TypeScript Pi agent reference. Do not modify it unless the user explicitly asks; it has its own `pi/AGENTS.md` rules.
- Active implementation is `rust-pi-agent/`, a beginner-friendly Rust port of Pi's core agent harness.
- Keep the Rust agent reusable as an engine for both the CLI and a future Tauri/Cursor-style desktop app; avoid baking terminal output into core logic when adding new subsystems.

## Read First

- Start with `ROADMAP.md`, `docs/ARCHITECTURE.md`, and `docs/PORTING_NOTES.md` before changing architecture or scope.
- For crate usage and provider setup, read `rust-pi-agent/README.md` and `rust-pi-agent/Cargo.toml`.
- Every Rust source file should keep its top module doc comment with `Ported from:` and `Simplifications:` notes.

## Rust Commands

- Work from `rust-pi-agent/` for Rust commands.
- Format: `cargo fmt`.
- Compile check: `cargo check`.
- Run CLI help: `cargo run -- --help`.
- Provider smoke test: `cargo run -- --check-provider`.
- Harmless one-shot smoke test: `cargo run -- "Say exactly: ok"`.
- Example verification: `cargo run --example fibonacci` currently prints `55`.

## Provider And Secrets

- Local provider config is loaded from `rust-pi-agent/.env` via `dotenvy`; `.env` is ignored and must not be committed.
- Current defaults target AveMujicaAPI: `RUST_PI_BASE_URL=https://api.avemujica.moe/v1`, `RUST_PI_MODEL=gpt-5.5`.
- `--check-provider` makes a real API call and requires `OPENAI_API_KEY`; do not run it unless provider behavior is being verified.

## Current Agent Capabilities

- The Rust agent currently supports OpenAI-compatible chat completions with tool calls.
- Implemented tools: `read`, `write`, `bash`.
- Planned next tools/features are `edit` with rich diffs, then `ls`, `grep`, `find`, slash-command skills, and JSONL sessions.

## Style And Architecture

- Prefer simple Rust: direct structs/enums/functions, clear `match`, `anyhow::Result` in app code.
- Keep `pi/` behavior as reference, but port behavior selectively; do not clone Pi's extension/package/OAuth/TUI complexity early.
- Before desktop work, add a `lib.rs` boundary and structured agent events so the CLI is only one frontend.
- For risky future desktop tools, plan approval flows: allow read-only tools by default; require approval for `write`, `edit`, and `bash`.
