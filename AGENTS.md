# Repository Instructions

## Scope

- Active implementation is `sabi-agent/`; run Rust commands from that directory unless intentionally testing another CWD.
- `pi/` is the original TypeScript Pi reference submodule. Do not modify it unless the user explicitly asks; it has its own `pi/AGENTS.md`.
- Keep core Rust agent logic reusable behind `src/lib.rs`; `src/main.rs` should stay a thin CLI frontend.

## Read First

- For current state and scope, read `README.md`, `sabi-agent/README.md`, and `ROADMAP.md`.
- For architecture changes, also read `docs/ARCHITECTURE.md` and `docs/PORTING_NOTES.md`.
- Every Rust source file should keep its top module doc comment with `Ported from:` and `Simplifications:` notes.

## Commands

- Format: `cargo fmt` from `sabi-agent/`.
- Compile check: `cargo check` from `sabi-agent/`.
- CLI help: `cargo run -- --help`.
- Interactive CLI: `cargo run`.
- Resume latest non-empty session for the current CWD: `cargo run -- --resume`.
- Harmless one-shot smoke test: `cargo run -- "Say exactly: ok"`.
- Provider smoke test: `cargo run -- --check-provider`; this makes real API calls and requires `OPENAI_API_KEY`.

## Provider And Config

- `.env` is loaded from the process current working directory via `dotenvy`; running with `--manifest-path` from another directory will not load `sabi-agent/.env`.
- API keys (`OPENAI_API_KEY`, `EXA_API_KEY`) must come from `.env` or environment variables only — never from config files.
- Presets (`model`, `base_url`) are loaded from config files in this order: `sabi.toml` (project) > `~/.sabi/config.toml` (user) > `RUST_PI_MODEL`/`RUST_PI_BASE_URL` (env) > defaults (`gpt-5.5` at AveMujicaAPI).
- Never commit `.env`, `sabi.toml`, or provider credentials.

## Current Capabilities

- Built-in tools are `read`, `write`, `edit`, `bash`, `ls`, `grep`, `find`, `web_search`, and `exa_search`.
- `web_search` and `exa_search` require `EXA_API_KEY` and use the Exa API directly (no MCP proxy).
- `grep` shells out to `rg`; `find` shells out to `fd`, so missing binaries are runtime tool errors.
- Interactive mode requires approval for `write`, `edit`, and `bash`; `/fiwb` or `/yolo` bypasses approvals for the current process only.
- One-shot prompt mode currently allows tools without interactive approval.
- JSONL sessions are append-only; `--resume` only loads sessions whose stored header `cwd` matches the current working directory.
- Built-in skills: `/skill:init` creates or updates repository `AGENTS.md`; `/skill:review` reviews code changes.
- Additional skills load from `.sabi/skills/` and `~/.sabi/skills/`; skill summaries are included in ordinary prompts.

## Architecture Constraints

- Agent work should emit structured events from core logic; keep terminal rendering in the CLI layer.
- File mutation tools should emit diff/file events so future desktop UI can render changes safely.
- Prefer simple Rust: direct structs/enums/functions, clear `match`, and `anyhow::Result` in app code.
- Port Pi behavior selectively; do not clone Pi's extension/package/OAuth/TUI/RPC complexity unless explicitly requested.
