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
- AveMujicaAPI defaults via environment variables or a working-directory `.env`.
- Tool calls for `read`, `write`, `edit`, `bash`, `ls`, `grep`, `find`, `web_search`, and `exa_search`.
- Structured agent events used by the CLI renderer.
- JSONL session files with `--resume` for the latest non-empty session in the current working directory.
- Interactive approvals for `write`, `edit`, and `bash`, with session-only `/fiwb` mode to bypass approvals.
- Skill discovery from `.sabi/skills` and `~/.sabi/skills`, with `/skill:name` invocation.
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

Current local `.env` keys:

```dotenv
OPENAI_API_KEY=...
RUST_PI_MODEL=gpt-5.5
RUST_PI_BASE_URL=https://api.avemujica.moe/v1
EXA_API_KEY=...
```

Do not commit `.env`; it is ignored.

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
