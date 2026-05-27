# Sabi Agent Roadmap

This project is a beginner-friendly Rust port of the core Pi agent harness ideas, now branded as Sabi Agent.
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

Status: complete.

Deliverables:
- Rust crate under `sabi-agent/`.
- CLI entry point.
- Module layout matching the learning roadmap.
- Documentation comments in every Rust source file.
- Basic `cargo check` passes.

## Milestone 2: Plain Chat Loop

Status: complete.

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

Status: complete for the initial `read`, `write`, and `bash` tool set.

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

Status: partially complete. `edit` is implemented and registered with exact replacement, uniqueness checks, and basic diff/file events. Rich terminal diff polish and intra-line highlighting are still pending.

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

Status: partially complete. `src/lib.rs` exists, the CLI uses the library crate, and the agent loop emits structured events for assistant text, tool lifecycle, diffs, and file changes. A fuller desktop-facing API can come later.

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

Status: complete for first-pass implementations. `ls` uses Rust filesystem APIs, `grep` shells out to `rg`, `find` shells out to `fd`, and `web_search`/`exa_search` use the Exa API directly. Output truncation exists for `grep` and `find`.

Deliverables:
- Add `ls` using Rust filesystem APIs.
- Add `grep` by shelling out to `rg`.
- Add `find` by shelling out to `fd`, with a fallback considered later.
- Add `web_search` for general web search via Exa API (requires `EXA_API_KEY`).
- Add `exa_search` for code/docs search via Exa API.
- Truncate large outputs with clear notices.

## Milestone 6: Slash Commands And Skills

Status: complete for the first pass. Slash commands exist, skills are discovered from `.sabi/skills/` and `~/.sabi/skills/`, `/skill:name optional extra instructions` invokes a loaded skill, and available skill summaries are included in ordinary prompts.

Deliverables:
- Add `/help`, `/quit`, `/clear`, `/new`, `/session`, `/reload`.
- Load skills from project and user skill directories.
- Include available skills in the system prompt.
- Invoke skills with `/skill:name optional extra instructions`.

Skill search locations:
- `.sabi/skills/`
- `~/.sabi/skills/`

## Milestone 7: JSONL Sessions

Status: partially complete. New runs create an append-only JSONL session file with a header, message entries, and optional metadata entries such as a desktop session title. `--resume` loads the latest non-empty session for the current working directory and continues appending to it. `/reload` can manually load the latest previous session into memory. CLI session selection is not implemented yet.

Deliverables:
- Save session headers and message entries to JSONL.
- Save append-only metadata entries such as session titles.
- Resume the most recent session.
- Keep the session model linear before adding branches.

Out of scope:
- Forking.
- Cloning.
- Tree navigation.
- Compaction.

## Milestone 8: Polish

Status: complete.

Deliverables:
- Better error messages (helpful hints for missing API keys, unknown tools, invalid skill frontmatter).
- Config file (`sabi.toml`) for model/base URL with per-project overrides.
- Test coverage for diff logic (3 unit tests for unified patch and terminal diff rendering).
- Readline history persisted across restarts.

## Milestone 9: Desktop App Foundation

Status: partially complete. The `desktop/` Tauri shell exists with native project selection, backend health, session listing, session title display, right-click session deletion, skill/file autocomplete, prompt execution, basic transcript rendering, and a cleaned-up minimal layout. Rich event streaming and polished diff rendering are still pending.

Next slice:
- Wire a Tauri-managed `DesktopAgent` instance for the selected workspace.
- Enable the prompt composer to send one user prompt through `DesktopAgent::send_prompt`.
- Render a plain transcript with user messages, assistant replies, tool lifecycle events, diffs, and errors.
- Render a compact approval card for risky mutation/shell tools and continue the agent turn after the user's decision.

Deliverables:
- Create a separate desktop app shell, likely under `desktop/`.
- Use Tauri for a lightweight Rust-backed desktop app.
- Use a web UI frontend. The current shell uses Vanilla TypeScript/Vite; React or Svelte can be introduced if UI complexity justifies it.
- Communicate with the Rust agent engine through Tauri commands or a local API layer.
- Render structured agent events in the UI.

First desktop features:
- Chat panel with assistant messages and tool calls.
- Workspace file tree.
- Monaco-based file viewer/editor.
- Rich diff viewer for proposed file changes.
- Tool approval buttons for risky tools such as `write`, `edit`, and `bash`.
- Session list, session title display, right-click delete, and resume.
- Native project picker for switching workspace roots.
- File, slash-command, and skill autocomplete in the prompt composer.

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
