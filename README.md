# Sabi Agent

A beginner-friendly Rust coding-agent harness.

This repository is a learning project. The goal is to understand and rebuild the essential parts of an agent harness in simple Rust while keeping the original TypeScript Pi agent available as a reference.

## Layout

- `sabi-agent/` - active Rust implementation.
- `pi/` - original TypeScript Pi agent reference, tracked as a submodule.
- `ROADMAP.md` - implementation milestones and scope decisions.
- `docs/ARCHITECTURE.md` - architecture notes, including the future desktop app direction.
- `docs/PORTING_NOTES.md` - notes on what is intentionally simplified from Pi.
- `AGENTS.md` - instructions for future OpenCode sessions.

## Current State

The Rust agent currently supports:

- OpenAI-compatible chat completions.
- AveMujicaAPI defaults via environment variables, a working-directory `.env`, or a `sabi.toml` config file.
- Tool calls for `read`, `write`, `edit`, `bash`, `ls`, `grep`, `find`, `web_search`, and `exa_search`.
- Structured agent events used by the CLI renderer.
- JSONL session files with `--resume` for the latest non-empty session in the current working directory.
- Interactive approvals for `write`, `edit`, and `bash`, with session-only `/fiwb` mode to bypass approvals.
- Skill discovery from `.sabi/skills` and `~/.sabi/skills`, with `/skill:name` invocation.
- Readline command history persisted across restarts.
- Unit tests for diff logic.
- Generated Fibonacci examples used to verify file tools.

Planned next:

- Session selection and richer resume UX.
- Richer approval UX for risky operations.
- A fuller desktop-facing API for a future Tauri/Cursor-style frontend.

## Setup

```bash
cd sabi-agent
cp .env.example .env
```

Current local `.env` keys (API keys only):

```dotenv
OPENAI_API_KEY=...
EXA_API_KEY=...
```

Presets (model, base URL) are loaded from config files — never from `.env`:

1. **Project-level**: `sabi.toml` in the working directory
2. **User-level**: `~/.sabi/config.toml`
3. **Environment**: `RUST_PI_MODEL`, `RUST_PI_BASE_URL`
4. **Defaults**: `gpt-5.5` at `https://api.avemujica.moe/v1`

Example `~/.sabi/config.toml`:

```toml
model = "gpt-5.5"
base_url = "https://api.avemujica.moe/v1"
```

Example per-project `sabi.toml`:

```toml
model = "gpt-4o-mini"
base_url = "https://api.openai.com/v1"
```

Do not commit `.env`, `sabi.toml`, or provider credentials.

`.env` is loaded from the process current working directory. If you run the binary from another directory, export the required variables explicitly or provide a `.env` in that directory.

## Commands

Run from `sabi-agent/`:

```bash
cargo fmt
cargo check
cargo run -- --help
cargo run -- --check-provider
cargo run -- --resume
cargo run -- "Say exactly: ok"
cargo run --example fibonacci
```

`--check-provider` makes a real provider call and requires `OPENAI_API_KEY`.

## Reference

The `pi/` submodule points at the original Pi agent. Keep it unchanged unless explicitly working on the reference implementation.
