# Porting Notes

This file records decisions made while translating Pi concepts into Rust.

## Keep Pi As Reference

The `pi/` directory is intentionally left unchanged. The Rust implementation should refer to Pi files by path in comments and docs, but should not modify or depend on the TypeScript source at runtime.

## Port Behavior, Not Framework Complexity

Pi has a mature architecture for multiple modes, extensions, SDK usage, RPC, package management, provider registries, and a custom TUI. The Rust version ports the core behavior only:

- Message loop.
- Tool calls.
- Tool results.
- File mutation tools.
- Diff display.
- Skills as prompt resources.
- Slash commands.
- JSONL sessions.

The Rust version should also keep the core agent code reusable for non-terminal frontends. A future Codex Desktop / Cursor-style app should use the same agent engine rather than a separate implementation.

## Beginner-Friendly Rust Choices

- Use `String`, `Vec`, `HashMap`, and enums directly.
- Prefer `anyhow::Result<T>` in application code.
- Avoid lifetimes in public structs unless required.
- Avoid trait objects until tool dispatch needs them.
- Avoid macros beyond common derives like `Debug`, `Clone`, `Serialize`, and `Deserialize`.
- Prefer clear `match` statements over dense iterator chains.
- Keep UI-specific code outside the core agent modules when possible.

## Tool Simplification Notes

`read` starts text-only. Pi supports images, model vision checks, syntax highlighting, and TUI rendering.

`write` creates parent directories and overwrites files. Pi also has richer TUI preview behavior.

`edit` uses exact replacement and uniqueness checks. Pi includes fuzzy matching and more complete diff utilities.

`bash` captures output and supports timeout. Pi also handles process groups, partial output updates, and shell environment details.

`ls` uses Rust filesystem APIs.

`grep` and `find` use external `rg` and `fd`. Pi can download missing tools; the Rust version currently reports a clear error if they are not installed.

`write` and `edit` emit diff/file events so frontends can render file changes without scraping text output.

## Skill Simplification Notes

Skills should follow the Agent Skills convention with `SKILL.md` and frontmatter. The first Rust version only needs:

- `name`
- `description`
- `disable-model-invocation`
- full markdown content

Ignore files and package-installed skills can come later.

## Desktop App Notes

The long-term target includes a desktop application similar in spirit to Codex Desktop or Cursor agents.

Design implications:

- Agent code should emit typed events instead of only printing strings.
- Tool calls should be represented as structured data that a UI can render.
- File mutations should produce paths and diffs so a UI can show apply/reject flows.
- Sessions should be stored in a frontend-neutral format such as JSONL.
- The CLI should remain useful, but it should be only one frontend over the shared agent engine.

Likely desktop stack:

- Tauri for the desktop shell.
- React or Svelte for the UI.
- Monaco Editor for code editing.
- `xterm.js` for terminal display if needed.
- Rust `notify` for file watching.

Avoid doing this too early. The desktop app should come after the tool loop, edit/diff flow, skills, and sessions are stable enough to expose through a library API.
