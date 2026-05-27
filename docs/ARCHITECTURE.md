# Sabi Agent Architecture

This document describes the intended structure of the Rust implementation in `sabi-agent/`.

## Repository Layout

```text
sabi-agent/
  pi/                 # Original TypeScript Pi agent reference. Do not modify.
  sabi-agent/         # Rust learning implementation.
  desktop/            # Future Tauri desktop frontend.
  ROADMAP.md          # Milestones and porting scope.
  docs/
    ARCHITECTURE.md   # This file.
```

## Rust Crate Layout

```text
sabi-agent/src/
  lib.rs              # Reusable library entry point.
  main.rs             # CLI entry point.
  app.rs              # Top-level application flow.
  agent.rs            # Agent loop and tool-call orchestration.
  llm.rs              # Provider HTTP integration.
  messages.rs         # Message and content data types.
  config.rs           # CLI/config/environment loading.
  diff.rs             # Patch generation and terminal diff rendering.
  skills.rs           # Skill discovery and invocation formatting.
  slash.rs            # Slash command parsing and handling.
  session.rs          # JSONL transcript persistence.
  desktop.rs          # Desktop-facing in-process API over sessions, prompts, approvals, and events.
  tools/
    mod.rs            # Tool registry and shared tool types.
    read.rs           # Read tool.
    write.rs          # Write tool.
    edit.rs           # Edit tool.
    bash.rs           # Bash tool.
    ls.rs             # Directory listing tool.
    grep.rs           # Ripgrep-backed search tool.
    find.rs           # fd-backed file finder.
    search.rs         # Exa web search and code search.
```

Reusable modules now live behind `lib.rs`, and `main.rs` is a thin CLI frontend. `desktop.rs` provides the first desktop-facing boundary so a future Tauri app can drive sessions and prompt turns without parsing terminal output.

## Core Flow

1. A frontend collects user input. Today that is the CLI; later it can be a desktop app.
2. The frontend loads config, sessions, skills, and tools.
3. User input is converted into a user message.
4. `agent.rs` sends the conversation and active tools to `llm.rs`.
5. `llm.rs` returns an assistant message.
6. If the assistant requested tools, `agent.rs` executes them and appends tool result messages.
7. The loop continues until the assistant returns no tool calls.
8. The agent emits structured events while it works.
9. `session.rs` appends each message to JSONL as it is created.
10. `--resume` can load the latest non-empty JSONL session whose stored `cwd` matches the current working directory.

This mirrors the high-level behavior in `pi/packages/agent/src/agent-loop.ts` while intentionally omitting Pi's extension hooks, compaction, branching, and multi-provider registry.

## Frontend Boundary

The agent engine should not permanently depend on terminal printing. Terminal output is useful for early learning, but a desktop application needs structured events.

Target event examples:

```rust
pub enum AgentEvent {
    AssistantText { text: String },
    ToolStarted { id: String, name: String, args: serde_json::Value },
    ToolFinished { id: String, name: String, output: String, is_error: bool },
    DiffReady { path: String, patch: String, rendered: String },
    FileChanged { path: String },
    Error { message: String },
}
```

The CLI can render these events as text. A desktop app can render them as chat bubbles, tool cards, diff panels, notifications, and approval prompts.

`desktop.rs` exposes `DesktopAgent` as the frontend handle. It can start or resume a session, resume a specific session id, list non-empty sessions newest-first, send a prompt with event and approval callbacks, clear or create sessions, reload the previous session, refresh skills, and return serializable state for UI headers/lists. Session files can also carry append-only metadata entries, currently used for desktop session titles.

## Desktop Architecture Target

The desktop app is a separate Tauri frontend over the same Rust agent engine.

Recommended stack:

- Tauri for desktop packaging and Rust integration.
- Vanilla TypeScript/Vite with Tailwind CSS for the current shell; React or Svelte can be introduced when UI complexity justifies it.
- Monaco Editor for Cursor-style file viewing and editing.
- `xterm.js` for an optional terminal panel.
- Rust `notify` crate for workspace file watching.
- Tauri commands or an internal local API for frontend/backend communication.

Target layout:

```text
desktop/
  src-tauri/          # Tauri shell that calls sabi-agent library code.
  src/                # Vite web UI.
    main.ts           # Minimal project/session/composer frontend.
    styles.css        # Tailwind component layers and shell styling.
    # Future: ChatPanel, FileTree, EditorPanel, DiffViewer, ToolCard.
```

The desktop app should not call the CLI binary and parse stdout. It should call Rust library functions or subscribe to typed event streams.

Current desktop shell capabilities:

- Native project directory selection through Tauri's dialog plugin.
- Backend health check and workspace-scoped session listing.
- Session titles from JSONL metadata, with fallback titles from the first user message.
- Right-click session deletion through a validated Tauri command.
- Prompt composer autocomplete for files, slash commands, and skills.
- Prompt execution through a Tauri-managed `DesktopAgent`.
- Compact approval cards for `write`, `edit`, and `bash`.
- Compact tool rows and collapsible diff rendering for structured events.

Pending desktop shell capabilities:

- Live event streaming instead of returning a completed event batch.
- Run cancellation while an agent turn is active.
- Richer editor/file-tree panes and side-by-side diffs.

## Tool Approval In Desktop Mode

The agent engine builds a `ToolApprovalRequest` before executing each tool. The request contains the tool call id, name, arguments, risk level, whether approval is required, and a concise summary for UI display.

Current policy:

- `read`, `ls`, `grep`, `find`: read-only, allow by default.
- `web_search`, `exa_search`: external network, allow by default for now.
- `write`, `edit`: file mutation, require approval.
- `bash`: shell execution, require approval with a clear command preview.

The CLI answers these requests with a terminal prompt unless `/fiwb` or `/yolo` is enabled. The desktop frontend renders the same typed request as an approval card and returns the user's decision through a Tauri command.

## Tool Design

Tools should start as simple async functions wrapped by a small registry.

The first version can use a trait like this:

```rust
#[async_trait::async_trait]
pub trait Tool {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn parameters_schema(&self) -> serde_json::Value;
    async fn run(&self, args: serde_json::Value, ctx: ToolContext) -> anyhow::Result<ToolOutput>;
}
```

If `async_trait` feels too magical while learning Rust, we can avoid the trait at first and dispatch with a `match` on tool name.

## Documentation Rule

Every Rust source file should start with a module doc comment:

```rust
//! One sentence explaining the file.
//!
//! Ported from:
//! - `pi/.../source.ts`
//!
//! Simplifications:
//! - What this Rust version intentionally omits for now.
```

This project is a learning port, so source attribution and simplification notes are part of the codebase design.

## Initial Dependency Strategy

Start with crates that are common, stable, and easy to understand:

- `anyhow` for application-level errors.
- `clap` for CLI parsing.
- `directories` for user config/session directories.
- `ignore` for `.gitignore`-aware traversal when needed.
- `owo-colors` for terminal colors.
- `reqwest` for provider HTTP calls.
- `rustyline` for interactive input history.
- `schemars` for JSON schema generation.
- `serde` and `serde_json` for data types.
- `similar` for text diffs.
- `time` for timestamps.
- `toml` for per-project config file parsing.
- `tokio` for async runtime and process execution.
- `tracing` and `tracing-subscriber` for diagnostics.
- `uuid` for session IDs.
- `which` for locating external tools.

Add later only when a feature needs them:

- `diffy` for unified patch convenience.
- `globset` for Rust-native glob matching.
- `command-group` for process-tree termination.
- `ratatui` and `crossterm` for a real TUI.
- `notify` for desktop file watching.
- Tauri dependencies when the `desktop/` frontend begins.

## Provider Strategy

The first provider should be OpenAI-compatible because it keeps the first agent loop small.

Minimum config:

- `~/.sabi/auth.toml` for API keys.
- `~/.sabi/config.toml` for user-level model and base URL presets.
- Environment variables (`OPENAI_API_KEY`, `RUST_PI_MODEL`, `RUST_PI_BASE_URL`) for process-local overrides.
- Optional `sabi.toml` in the working directory for per-project model/base URL overrides.

Anthropic and provider abstraction can come after the basic tool loop works.
