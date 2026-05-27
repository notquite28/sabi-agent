# Sabi Agent

This crate is a small Rust coding-agent harness.

The original TypeScript implementation remains in `../pi/` for reference. This crate does not modify or depend on that code at runtime.

See:
- `../docs/USER_MANUAL.md`
- `../ROADMAP.md`
- `../docs/ARCHITECTURE.md`
- `../docs/PORTING_NOTES.md`

## Current State

The crate currently supports an OpenAI-compatible agent loop with these tools:

- `read`
- `write`
- `edit`
- `bash`
- `ls`
- `grep`
- `find`
- `web_search`
- `exa_search`

It also supports:

- Structured agent events rendered by the CLI.
- JSONL session files.
- `--resume` for the latest non-empty session whose stored `cwd` matches the current working directory.
- Interactive approval prompts for `write`, `edit`, and `bash`.
- `/fiwb` mode to allow risky tools for the current process only. It resets after restart.
- Skill discovery and `/skill:name optional instructions` invocation.
- Readline command history persisted across restarts.
- First-launch onboarding that guides you through preset configuration.
- Unit tests for diff logic.

## First Launch

On your first run, Sabi Agent will detect that you have no `~/.sabi/config.toml` and guide you through:

1. Setting your default model and base URL presets (saved to `~/.sabi/config.toml`)
2. Storing your API keys (saved to `~/.sabi/auth.toml` with restricted permissions)
3. Understanding the `~/.sabi/` directory layout

You can skip this setup and configure manually later by editing files in `~/.sabi/`.

## The ~/.sabi/ Directory

Everything lives in one place:

```
~/.sabi/
  config.toml    – presets (model, base_url)
  auth.toml      – API keys (600 permissions, owner-only)
  sessions/      – conversation JSONL files
  history        – command history
```

## Configuration

### API Keys

API keys are loaded in this order of precedence:

1. **User auth file**: `~/.sabi/auth.toml` created during onboarding with 600 permissions
2. **Environment variables**: `OPENAI_API_KEY`, `EXA_API_KEY` for process-local overrides

Example `~/.sabi/auth.toml`:

```toml
openai_api_key = "sk-..."
exa_api_key = "your-exa-key"
```

### Presets (Model, Base URL)

Presets are loaded in this order of precedence:

1. **Project-level**: `sabi.toml` in the working directory
2. **User-level**: `~/.sabi/config.toml`
3. **Environment**: `RUST_PI_MODEL`, `RUST_PI_BASE_URL`
4. **Defaults**: `gpt-5.5` at `https://api.avemujica.moe/v1`

Example `~/.sabi/config.toml` (both naming styles work):

```toml
model = "gpt-5.5"
base_url = "https://api.avemujica.moe/v1"

# Or use the env var names:
# RUST_PI_MODEL = "gpt-5.5"
# RUST_PI_BASE_URL = "https://api.avemujica.moe/v1"
```

Example per-project `sabi.toml`:

```toml
model = "gpt-4o-mini"
base_url = "https://api.openai.com/v1"
```

Invalid config files print a warning but do not crash.

## Run

```bash
cargo run -- --help
cargo run -- --check-provider
cargo run -- --resume
cargo run -- "Say exactly: ok"
cargo run -- "Read README.md and summarize it"
cargo run
```

`--check-provider` verifies:

- `RUST_PI_BASE_URL/v1/models` is reachable through the configured base URL.
- `RUST_PI_MODEL` exists in the returned model list.
- `RUST_PI_BASE_URL/v1/chat/completions` accepts a minimal request for the selected model.

## Sessions

Interactive runs create append-only JSONL session files under the user data directory, grouped by working-directory name. Each file stores a header with the original `cwd` and message entries as the conversation progresses.

Use `cargo run -- --resume` to resume the latest non-empty session for the current working directory. Sessions from other working directories are ignored.

Use `/clear` to clear only the in-memory conversation (system prompt is re-injected). Use `/new` to clear the conversation and start a fresh session file (system prompt is re-injected). Use `/reload` to load the previous session and re-inject the system prompt.

## Tool Approval

In interactive mode, `write`, `edit`, and `bash` require approval before execution. Read-only tools such as `read`, `ls`, `grep`, and `find` run without approval.

Use `/fiwb` to toggle session-only "Fuck it we ball" mode. `/yolo` is accepted as an alias. While enabled, risky tools run without approval. The mode is in memory only and resets when the process exits.

## Skills

The agent includes a built-in `init` skill for creating or updating repository `AGENTS.md` files.

Additional skills are loaded from `.sabi/skills/` and `~/.sabi/skills/`. Each skill is a `SKILL.md` file with `name` and `description` frontmatter.

Use `/skill:name optional extra instructions` to invoke a loaded skill. `/reload` reloads skill definitions after loading the previous session.

Loaded skill names and descriptions are included in ordinary prompts so the model can suggest `/skill:name` when a skill looks relevant.
