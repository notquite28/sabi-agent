# Sabi Agent

A beginner-friendly Rust coding-agent harness.

This repository is a learning project. The goal is to understand and rebuild the essential parts of an agent harness in simple Rust while keeping the original TypeScript Pi agent available as a reference.

## Layout

- `sabi-agent/` - active Rust implementation.
- `pi/` - original TypeScript Pi agent reference, tracked as a submodule.
- `ROADMAP.md` - implementation milestones and scope decisions.
- `docs/ARCHITECTURE.md` - architecture notes, including the future desktop app direction.
- `docs/USER_MANUAL.md` - CLI user manual covering setup, commands, sessions, approvals, tools, and skills.
- `docs/PORTING_NOTES.md` - notes on what is intentionally simplified from Pi.
- `AGENTS.md` - instructions for future OpenCode sessions.

## Current State

The Rust agent currently supports:

- OpenAI-compatible chat completions.
- AveMujicaAPI defaults from `~/.sabi/config.toml`, environment overrides, or a working-directory `sabi.toml`.
- Tool calls for `read`, `write`, `edit`, `bash`, `ls`, `grep`, `find`, `web_search`, and `exa_search`.
- Structured agent events used by the CLI renderer.
- JSONL session files with `--resume` for the latest non-empty session in the current working directory.
- Interactive approvals for `write`, `edit`, and `bash`, with session-only `/fiwb` mode to bypass approvals.
- Skill discovery from `.sabi/skills` and `~/.sabi/skills`, with `/skill:name` invocation.
- Readline command history persisted across restarts.
- First-launch onboarding that guides you through preset configuration.
- Unit tests for diff logic.
- Generated Fibonacci examples used to verify file tools.

Planned next:

- Session selection and richer resume UX.
- Richer approval UX for risky operations.
- A fuller desktop-facing API for a future Tauri/Cursor-style frontend.

## Setup

On first launch, Sabi Agent will guide you through setting up default presets (model, base URL) and API keys under `~/.sabi/`. You can also skip this and configure manually later.

Everything lives in `~/.sabi/`:

```
~/.sabi/
  config.toml    – presets (model, base_url)
  auth.toml      – API keys (600 permissions, owner-only)
  sessions/      – conversation JSONL files
  history        – command history
```

**API keys** are loaded in this order:
1. `~/.sabi/auth.toml` (created during onboarding)
2. Environment variables (`OPENAI_API_KEY`, `EXA_API_KEY`) for process-local overrides

**Presets** (model, base URL) are loaded in this order:
1. `sabi.toml` in working directory
2. `~/.sabi/config.toml`
3. `RUST_PI_MODEL` / `RUST_PI_BASE_URL` env vars
4. Defaults: `gpt-5.5` at `https://api.avemujica.moe/v1`

Do not commit `sabi.toml` or provider credentials.

## Commands

Run from `sabi-agent/`.

For detailed usage, see the [CLI user manual](docs/USER_MANUAL.md).

```bash
cargo fmt
cargo check
cargo run -- --help
cargo run -- --check-provider
cargo run -- --resume
cargo run -- "Say exactly: ok"
cargo run --example fibonacci
```

`--check-provider` makes a real provider call and requires `openai_api_key` in `~/.sabi/auth.toml` or `OPENAI_API_KEY` in the environment.

## Reference

The `pi/` submodule points at the original Pi agent. Keep it unchanged unless explicitly working on the reference implementation.
