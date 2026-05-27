# Sabi Agent CLI User Manual

Sabi Agent is a small Rust coding-agent harness. It runs from the terminal, talks to an OpenAI-compatible chat-completions provider, and can use local file, shell, search, session, and skill features while it works.

Run commands from the Rust crate directory unless you have installed the binary elsewhere:

```bash
cd sabi-agent
```

## Requirements

- Rust toolchain with `cargo`.
- An OpenAI-compatible API key, provided either in `~/.sabi/auth.toml` or as `OPENAI_API_KEY`.
- `ripgrep` (`rg`) if you want the agent's `grep` tool to work.
- `fd` if you want the agent's `find` tool to work.
- Optional: an Exa API key, provided either in `~/.sabi/auth.toml` as `exa_api_key` or as `EXA_API_KEY`, for `web_search` and `exa_search`.

## First launch

Start the CLI:

```bash
cargo run
```

If `~/.sabi/config.toml` does not exist, Sabi Agent starts onboarding. It can create:

```text
~/.sabi/
  config.toml    # model and base_url presets
  auth.toml      # API keys, written with owner-only permissions on Unix
  sessions/      # JSONL conversation sessions
  history        # interactive command history
```

During onboarding, press Enter to accept the defaults or enter your own model, base URL, and API keys. You can skip setup and edit these files manually later.

## Configuration

### API keys

Sabi Agent loads the provider API key in this order:

1. `openai_api_key` from `~/.sabi/auth.toml`
2. `OPENAI_API_KEY` from the process environment

Example `~/.sabi/auth.toml`:

```toml
openai_api_key = "sk-..."
exa_api_key = "exa-..."
```

The Exa key is optional. It is only required when the agent calls `web_search` or `exa_search`.

### Model and base URL

Sabi Agent loads model presets in this order:

1. `sabi.toml` in the current working directory
2. `~/.sabi/config.toml`
3. `RUST_PI_MODEL` and `RUST_PI_BASE_URL` environment variables
4. Defaults: `gpt-5.5` and `https://api.avemujica.moe/v1`

Example `~/.sabi/config.toml` or project-local `sabi.toml`:

```toml
model = "gpt-5.5"
base_url = "https://api.avemujica.moe/v1"
```

Do not commit project-local `sabi.toml` files if they contain private provider details.

## CLI commands

Show the built-in help:

```bash
cargo run -- --help
```

Current help output:

```text
A small Rust coding agent harness

Usage: sabi-agent [OPTIONS] [PROMPT]...

Arguments:
  [PROMPT]...  Optional one-shot prompt. If omitted, interactive mode starts

Options:
      --check-provider  Check provider connectivity, selected model availability, and chat support
      --resume          Resume the latest non-empty session for the current working directory
  -h, --help            Print help
```

### Interactive mode

Start an interactive session:

```bash
cargo run
```

You will see the session file path and a prompt:

```text
sabi-agent ready with read/write/edit/bash/ls/grep/find tools. Type /help for commands or /quit to exit.
session: ~/.sabi/sessions/<workspace>/<session-id>.jsonl
> 
```

Type ordinary requests at the `>` prompt:

```text
> Read README.md and summarize the setup steps.
> Add a small example to the crate README.
> Run cargo check and fix any errors.
```

Interactive command history is saved in `~/.sabi/history` and loaded on the next run.

### One-shot prompt mode

Pass a prompt after `--` to run one agent turn and exit:

```bash
cargo run -- "Say exactly: ok"
cargo run -- "Read README.md and list the commands a new user should run"
```

One-shot mode creates a session file and appends the conversation, but it does not enter the interactive prompt loop.

Important: current one-shot mode auto-approves tool calls for that turn. Use interactive mode when you want approval prompts before `write`, `edit`, or `bash`.

### Resume the latest session

Resume the latest non-empty session whose stored `cwd` matches the current working directory:

```bash
cargo run -- --resume
```

If no matching non-empty session exists, Sabi Agent starts a new session.

### Check provider connectivity

Verify the configured provider and model:

```bash
cargo run -- --check-provider
```

This makes real network calls. It checks that:

- the configured base URL's models endpoint is reachable,
- the configured model exists in that model list,
- chat completions accepts a minimal request for that model.

## Slash commands

Slash commands are available only in interactive mode.

| Command | What it does |
| --- | --- |
| `/help` | Print the available slash commands. |
| `/quit` or `/exit` | Exit the interactive session. |
| `/clear` | Clear the in-memory conversation and re-inject the system prompt. The existing session file remains on disk. |
| `/new` | Clear the in-memory conversation and create a new session file. |
| `/session` | Show message count, session id, session file, loaded skill count, and FIWB mode state. |
| `/reload` | Load the latest previous non-empty session for this working directory, excluding the current session, and rediscover skills. |
| `/fiwb` or `/yolo` | Toggle session-only approval bypass for risky tools. It resets when the process exits. |
| `/skill:name optional instructions` | Invoke a loaded skill by name, optionally passing extra instructions. |

## Tool approvals

In interactive mode, Sabi Agent requires approval before these risky tools run:

- `write`
- `edit`
- `bash`

When approval is required, the CLI prints a short summary and asks:

```text
approval required: edit path="src/lib.rs"
Approve? [y/N]
```

Only `y`, `Y`, `yes`, or `YES` approves the tool call. Any other answer denies it.

Use `/fiwb` or `/yolo` to bypass these approvals for the current process only:

```text
> /fiwb
FIWB mode enabled for this process only. Risky tools will run without approval until restart or /fiwb.
```

Run `/fiwb` again to disable the bypass.

## Built-in agent tools

These tools are exposed to the model during an agent turn. You do not call them directly from the terminal; ask Sabi Agent to perform work and it chooses the needed tools.

| Tool | Capability | Notes |
| --- | --- | --- |
| `read` | Read a UTF-8 text file. | Supports optional 1-indexed `offset` and `limit`; default limit is 200 lines. |
| `write` | Create or overwrite a UTF-8 text file. | Creates parent directories. Emits file-change events and a diff when overwriting changed content. Requires approval in interactive mode. |
| `edit` | Replace one exact, unique text snippet in a UTF-8 text file. | Rejects empty `old_text`, no-op replacements, missing snippets, and non-unique snippets. Requires approval in interactive mode. |
| `bash` | Run a shell command in the current working directory. | Supports optional timeout seconds. Returns combined stdout and stderr plus exit status. Requires approval in interactive mode. |
| `ls` | List files and directories in a directory. | Defaults to the current working directory. Directories end with `/`. |
| `grep` | Search file contents using `rg`. | Supports regex pattern, optional path, optional include glob, and result limit. Requires `ripgrep`. |
| `find` | Find files by name pattern using `fd`. | Supports optional pattern, path, and result limit. Requires `fd`. |
| `web_search` | Search the web through Exa. | Requires `EXA_API_KEY` or `exa_api_key`. Defaults to 5 results, max 20. |
| `exa_search` | Search for code examples, documentation, and API references through Exa. | Requires `EXA_API_KEY` or `exa_api_key`. Defaults to 10 results, max 20. |

## Sessions

Sabi Agent stores conversations as append-only JSONL files under:

```text
~/.sabi/sessions/<workspace-name>/<session-id>.jsonl
```

Each session starts with a header containing the session id, creation time, and original working directory. Message entries are appended as the conversation progresses. Desktop-facing metadata entries, such as session titles, are also append-only JSONL entries so existing transcripts do not need to be rewritten.

Session behavior:

- A normal interactive run creates a new session file.
- `--resume` loads the latest non-empty session whose stored `cwd` matches the current working directory.
- `/reload` loads the latest previous non-empty session for the same working directory, excluding the session currently open in the CLI.
- `/clear` only clears memory for the current process; it does not delete the session file.
- `/new` creates a fresh session file and continues there.
- The desktop shell shows session titles when metadata exists, falls back to the first user message, and finally falls back to the short session id.
- In the desktop shell, right-click a session in the left pane and choose `Delete Session` to remove that session file after confirmation.

## Desktop shell

The early Tauri desktop shell lives in `desktop/`. It currently provides a lightweight project/session frontend over the Rust library, not a complete chat UI yet.

Run it from the desktop directory:

```bash
cd desktop
npm run tauri:dev
```

Current desktop features:

- Backend health indicator.
- Native `Open Project` directory picker.
- Workspace-scoped session list with persisted/fallback titles.
- Right-click session deletion.
- Prompt composer autocomplete for `@` files, `/` slash commands, and `/skill:name` skills.

Still pending:

- Sending prompts from the desktop composer.
- Rendering agent events as chat/tool/diff cards.
- Desktop approval prompts for risky tools.

## Skills

Sabi Agent always includes two built-in skills:

- `init` - create or update a compact repository `AGENTS.md` instruction file.
- `review` - review code changes and provide actionable feedback.

Additional skills are discovered from:

```text
.sabi/skills/
~/.sabi/skills/
```

A skill is a `SKILL.md` file with YAML-style frontmatter:

```markdown
---
name: example
description: Explain how to use this repository.
---

Use these instructions when the user asks for repository usage help.
```

Invoke a skill in interactive mode:

```text
> /skill:review focus on unsafe file operations
> /skill:init keep it short
```

If a skill file sets `disable-model-invocation: true`, the CLI refuses direct `/skill:name` invocation for that skill.

## Common workflows

### Start a new project session

```bash
cd sabi-agent
cargo run
```

Then ask:

```text
> Read README.md and explain the current project status.
```

### Make a small code change with approvals

```bash
cargo run
```

Then ask for the change. Review each `write`, `edit`, or `bash` approval prompt before approving.

### Continue yesterday's work

```bash
cargo run -- --resume
```

Then inspect state:

```text
> /session
```

### Use search-capable prompts

Add an Exa key first:

```toml
# ~/.sabi/auth.toml
openai_api_key = "sk-..."
exa_api_key = "exa-..."
```

Then ask Sabi Agent for current external information:

```text
> Search the web for the current Exa API docs and summarize authentication.
> Find code examples for using reqwest JSON POST requests in Rust.
```

## Troubleshooting

### Missing OpenAI API key

If startup prints that setup is incomplete, add a key to `~/.sabi/auth.toml`:

```toml
openai_api_key = "sk-..."
```

Or export it for just the current shell:

```bash
export OPENAI_API_KEY="sk-..."
cargo run
```

### Provider check fails

Run:

```bash
cargo run -- --check-provider
```

Then verify `model` and `base_url` in `sabi.toml` or `~/.sabi/config.toml`. The base URL should be the provider API root expected by Sabi Agent's OpenAI-compatible client.

### `grep` or `find` tool errors

Install the external tool named in the error:

- `grep` requires `rg` from ripgrep.
- `find` requires `fd`.

### No session resumes

`--resume` only considers sessions with message entries whose stored `cwd` exactly matches the directory you launched from. Run from the same project directory where the session was created.
