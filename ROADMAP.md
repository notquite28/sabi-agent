# Rust Pi Agent Roadmap

This project is a beginner-friendly Rust port of the core Pi agent harness ideas.
The original TypeScript Pi agent stays in `pi/` as the reference implementation.

The goal is not to clone Pi feature-for-feature. The goal is to learn how an agent harness works while keeping the Rust code small, direct, and easy to reason about.

## Guiding Principles

- Keep the original `pi/` directory unchanged.
- Prefer simple Rust syntax over clever abstractions.
- Add top-of-file module docs to every Rust file explaining what the file does and which Pi files inspired it.
- Build one working feature at a time.
- Use direct structs, enums, and functions before traits or generics.
- Shell out to proven tools like `rg` and `fd` before reimplementing complex search logic.
- Avoid full TUI complexity until the agent loop and tools are stable.
- Design the Rust agent as a reusable engine, not only a terminal program, so it can later power a Codex Desktop / Cursor-style app.

## Reference Map

Core agent loop:
- `pi/packages/agent/src/agent-loop.ts`
- `pi/packages/agent/src/agent.ts`
- `pi/packages/agent/src/types.ts`

Coding tools:
- `pi/packages/coding-agent/src/core/tools/index.ts`
- `pi/packages/coding-agent/src/core/tools/read.ts`
- `pi/packages/coding-agent/src/core/tools/write.ts`
- `pi/packages/coding-agent/src/core/tools/edit.ts`
- `pi/packages/coding-agent/src/core/tools/edit-diff.ts`
- `pi/packages/coding-agent/src/core/tools/bash.ts`
- `pi/packages/coding-agent/src/core/tools/ls.ts`
- `pi/packages/coding-agent/src/core/tools/grep.ts`
- `pi/packages/coding-agent/src/core/tools/find.ts`

Diff viewer:
- `pi/packages/coding-agent/src/modes/interactive/components/diff.ts`
- `pi/packages/coding-agent/src/core/tools/edit-diff.ts`

Skills and slash commands:
- `pi/packages/coding-agent/src/core/skills.ts`
- `pi/packages/agent/src/harness/skills.ts`
- `pi/packages/coding-agent/src/core/slash-commands.ts`

Sessions:
- `pi/packages/agent/src/harness/session/session.ts`
- `pi/packages/agent/src/harness/session/jsonl-storage.ts`

## Milestone 1: Project Skeleton

Status: in progress.

Deliverables:
- Rust crate under `rust-pi-agent/`.
- CLI entry point.
- Module layout matching the learning roadmap.
- Documentation comments in every Rust source file.
- Basic `cargo check` passes.

## Milestone 2: Plain Chat Loop

Deliverables:
- Read user input interactively.
- Send messages to a single OpenAI-compatible provider.
- Print assistant text responses.
- Store transcript in memory.

Out of scope:
- Tool calls.
- Streaming UI polish.
- Multiple providers.
- OAuth/login.

## Milestone 3: Agent Loop With Tools

Deliverables:
- Represent messages, assistant tool calls, and tool results.
- Register tools in a simple map.
- Send tool definitions to the provider.
- Execute requested tools.
- Append tool results and continue until the assistant stops requesting tools.

Initial tools:
- `read`
- `write`
- `bash`

## Milestone 4: File Editing And Rich Diffs

Deliverables:
- Add `edit` with exact text replacement.
- Reject missing, duplicate, empty, or no-op replacements.
- Generate a unified patch.
- Render a readable terminal diff with colors and line numbers.
- Add basic intra-line highlighting for single-line replacements.

Out of scope:
- Pi's fuzzy Unicode normalization at first.
- TUI boxes/collapsible output.

## Milestone 4.5: Library Boundary For Future Desktop UI

Deliverables:
- Add `src/lib.rs` and move reusable agent code behind a library API.
- Keep `src/main.rs` as a thin CLI wrapper around the library.
- Introduce structured agent events instead of printing directly from the agent loop.
- Represent events such as assistant text, tool start, tool finish, diff ready, file changed, and error.
- Keep the terminal CLI as the first frontend while making a desktop frontend possible later.

Why this matters:
- A Codex Desktop / Cursor-style app should not scrape terminal output.
- The same agent engine should support CLI, desktop UI, tests, and future automation.
- Tool approval, diffs, and file edits need structured data so a GUI can render them safely.

## Milestone 5: Search And Listing Tools

Deliverables:
- Add `ls` using Rust filesystem APIs.
- Add `grep` by shelling out to `rg`.
- Add `find` by shelling out to `fd`, with a fallback considered later.
- Truncate large outputs with clear notices.

## Milestone 6: Slash Commands And Skills

Deliverables:
- Add `/help`, `/quit`, `/clear`, `/session`, `/reload`.
- Load skills from project and user skill directories.
- Include available skills in the system prompt.
- Invoke skills with `/skill:name optional extra instructions`.

Skill search locations:
- `.agents/skills/`
- `.pi/skills/`
- `~/.agents/skills/`
- `~/.pi/agent/skills/`

## Milestone 7: JSONL Sessions

Deliverables:
- Save session headers and message entries to JSONL.
- Resume the most recent session.
- Keep the session model linear before adding branches.

Out of scope:
- Forking.
- Cloning.
- Tree navigation.
- Compaction.

## Milestone 8: Polish

Deliverables:
- Better error messages.
- Config file for model/base URL.
- Test coverage for tools and diff logic.
- Optional readline history.

## Milestone 9: Desktop App Foundation

Deliverables:
- Create a separate desktop app shell, likely under `desktop/`.
- Use Tauri for a lightweight Rust-backed desktop app.
- Use a web UI frontend, likely React or Svelte.
- Communicate with the Rust agent engine through Tauri commands or a local API layer.
- Render structured agent events in the UI.

First desktop features:
- Chat panel with assistant messages and tool calls.
- Workspace file tree.
- Monaco-based file viewer/editor.
- Rich diff viewer for proposed file changes.
- Tool approval buttons for risky tools such as `write`, `edit`, and `bash`.
- Session list and resume.

Later desktop features:
- Side-by-side file diffs.
- Inline apply/reject controls.
- Integrated terminal panel using `xterm.js`.
- File watching with Rust `notify`.
- Multiple concurrent sessions.
- Project-level settings UI.

## Explicitly Not Porting Yet

- TypeScript extension runtime.
- Pi package install/update/config commands.
- Full provider registry.
- OAuth subscription login flows.
- Image input/output.
- Themes and full TUI system.
- Desktop UI during the initial agent/tool milestones.
- RPC mode.
- SDK embedding API.
- HTML export/share.
- Session branching, tree navigation, clone/fork.
- Automatic compaction and branch summarization.
- Permission popups or sandboxing.
- Sub-agents and plan mode.
