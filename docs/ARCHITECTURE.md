# Rust Pi Agent Architecture

This document describes the intended structure of the Rust implementation in `rust-pi-agent/`.

## Repository Layout

```text
rust-agent/
  pi/                 # Original TypeScript Pi agent reference. Do not modify.
  rust-pi-agent/      # Rust learning implementation.
  desktop/            # Future Tauri desktop frontend.
  ROADMAP.md          # Milestones and porting scope.
  docs/
    ARCHITECTURE.md   # This file.
```

## Rust Crate Layout

```text
rust-pi-agent/src/
  lib.rs              # Future reusable library entry point.
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
  tools/
    mod.rs            # Tool registry and shared tool types.
    read.rs           # Read tool.
    write.rs          # Write tool.
    edit.rs           # Edit tool.
    bash.rs           # Bash tool.
    ls.rs             # Directory listing tool.
    grep.rs           # Ripgrep-backed search tool.
    find.rs           # fd-backed file finder.
```

The current crate may start as a binary-only project while learning. Before building the desktop app, move reusable modules behind `lib.rs` and keep `main.rs` as a thin CLI frontend.

## Core Flow

1. A frontend collects user input. Today that is the CLI; later it can be a desktop app.
2. The frontend loads config, sessions, skills, and tools.
3. User input is converted into a user message.
4. `agent.rs` sends the conversation and active tools to `llm.rs`.
5. `llm.rs` returns an assistant message.
6. If the assistant requested tools, `agent.rs` executes them and appends tool result messages.
7. The loop continues until the assistant returns no tool calls.
8. The agent emits structured events while it works.
9. `session.rs` appends each message to JSONL.

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

## Desktop Architecture Target

The future desktop app should be a separate frontend over the same Rust agent engine.

Recommended stack:

- Tauri for desktop packaging and Rust integration.
- React or Svelte for the UI.
- Monaco Editor for Cursor-style file viewing and editing.
- `xterm.js` for an optional terminal panel.
- Rust `notify` crate for workspace file watching.
- Tauri commands or an internal local API for frontend/backend communication.

Target layout:

```text
desktop/
  src-tauri/          # Tauri shell that calls rust-pi-agent library code.
  src/                # Web UI.
    ChatPanel.tsx
    FileTree.tsx
    EditorPanel.tsx
    DiffViewer.tsx
    ToolCard.tsx
```

The desktop app should not call the CLI binary and parse stdout. It should call Rust library functions or subscribe to typed event streams.

## Tool Approval In Desktop Mode

The CLI currently executes tools directly. A desktop app should eventually support approval before risky operations.

Suggested policy:

- `read`, `ls`, `grep`, `find`: allow by default.
- `write`, `edit`: show diff or target path and require approval.
- `bash`: require approval by default, with a clear command preview.

This can be implemented as a pre-tool hook in the agent engine that asks the frontend for a decision.

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

- `OPENAI_API_KEY`
- `RUST_PI_MODEL`, defaulting to a configurable model string.
- `RUST_PI_BASE_URL`, defaulting to the OpenAI API base URL.

Anthropic and provider abstraction can come after the basic tool loop works.
