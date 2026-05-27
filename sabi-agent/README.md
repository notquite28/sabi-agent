# Sabi Agent

This crate is a small Rust coding-agent harness.

The original TypeScript implementation remains in `../pi/` for reference. This crate does not modify or depend on that code at runtime.

See:
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

1. Understanding where API keys belong (`.env` or environment variables)
2. Setting your default model and base URL presets
3. Creating your user-level config file

You can skip this setup and configure manually later by editing `~/.sabi/config.toml`.

## Configuration

### API Keys (Required)

API keys must come from environment variables or a `.env` file in the working directory. They are never stored in config files.

```bash
export OPENAI_API_KEY=...
export EXA_API_KEY=...
```

Or in a `.env` file:

```dotenv
OPENAI_API_KEY=...
EXA_API_KEY=...
```

### Presets (Optional)

Model and base URL presets are loaded from config files in this order of precedence:

1. **Project-level**: `sabi.toml` in the working directory
2. **User-level**: `~/.sabi/config.toml`
3. **Environment**: `RUST_PI_MODEL`, `RUST_PI_BASE_URL`
4. **Defaults**: `gpt-5.5` at `https://api.avemujica.moe/v1`

Example `~/.sabi/config.toml` (user-level defaults):

```toml
model = "gpt-5.5"
base_url = "https://api.avemujica.moe/v1"
```

Example per-project `sabi.toml` (overrides user-level):

```toml
model = "gpt-4o-mini"
base_url = "https://api.openai.com/v1"
```

AveMujicaAPI docs:

- Base URL: `https://api.avemujica.moe/v1`
- Model examples: `gpt-5.5`, or any exact model ID available to your API key
- Model list endpoint: `https://api.avemujica.moe/v1/models`

Invalid config files print a warning but do not crash.

`.env` is loaded from the process current working directory. Running with `--manifest-path` from another directory will not automatically load `sabi-agent/.env`; export the variables explicitly if needed.

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
