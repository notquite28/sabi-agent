# Repository Instructions

## Scope

- Active implementation is `rust-pi-agent/`; run Rust commands from that directory unless intentionally testing another CWD.
- `pi/` is the original TypeScript Pi reference submodule. Do not modify it unless the user explicitly asks; it has its own `pi/AGENTS.md`.
- Keep core Rust agent logic reusable behind `src/lib.rs`; `src/main.rs` should stay a thin CLI frontend.

## Read First

- For current state and scope, read `README.md`, `rust-pi-agent/README.md`, and `ROADMAP.md`.
- For architecture changes, also read `docs/ARCHITECTURE.md` and `docs/PORTING_NOTES.md`.
- Every Rust source file should keep its top module doc comment with `Ported from:` and `Simplifications:` notes.

## Commands

- Format: `cargo fmt` from `rust-pi-agent/`.
- Compile check: `cargo check` from `rust-pi-agent/`.
- CLI help: `cargo run -- --help`.
- Interactive CLI: `cargo run`.
- Resume latest non-empty session for the current CWD: `cargo run -- --resume`.
- Harmless one-shot smoke test: `cargo run -- "Say exactly: ok"`.
- Provider smoke test: `cargo run -- --check-provider`; this makes real API calls and requires `OPENAI_API_KEY`.

## Provider And Config

- `.env` is loaded from the process current working directory via `dotenvy`; running with `--manifest-path` from another directory will not load `rust-pi-agent/.env`.
- Expected local env keys are `OPENAI_API_KEY`, `RUST_PI_MODEL`, and `RUST_PI_BASE_URL`; defaults target AveMujicaAPI model `gpt-5.5` at `https://api.avemujica.moe/v1`.
- Never commit `.env` or provider credentials.

## Current Capabilities

- Built-in tools are `read`, `write`, `edit`, `bash`, `ls`, `grep`, and `find`.
- `grep` shells out to `rg`; `find` shells out to `fd`, so missing binaries are runtime tool errors.
- Interactive mode requires approval for `write`, `edit`, and `bash`; `/fiwb` or `/yolo` bypasses approvals for the current process only.
- One-shot prompt mode currently allows tools without interactive approval.
- JSONL sessions are append-only; `--resume` only loads sessions whose stored header `cwd` matches the current working directory.

## Architecture Constraints

- Agent work should emit structured events from core logic; keep terminal rendering in the CLI layer.
- File mutation tools should emit diff/file events so future desktop UI can render changes safely.
- Prefer simple Rust: direct structs/enums/functions, clear `match`, and `anyhow::Result` in app code.
- Port Pi behavior selectively; do not clone Pi's extension/package/OAuth/TUI/RPC complexity unless explicitly requested.
