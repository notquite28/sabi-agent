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

`web_search` and `exa_search` use the Exa API directly (no MCP proxy). They require an `EXA_API_KEY` environment variable. `web_search` tries `/answer` first for a synthesized response, then falls back to `/search`. `exa_search` enriches queries with code-focused terms and uses the same Exa API.

`write` and `edit` emit diff/file events so frontends can render file changes without scraping text output.

## Config Simplification Notes

The Rust version supports four config sources, in order of precedence:

1. `sabi.toml` in the working directory (per-project overrides for `model` and `base_url`).
2. `~/.sabi/config.toml` in the user's home directory (user-level defaults).
3. Environment variables (`RUST_PI_MODEL`, `RUST_PI_BASE_URL`).
4. Hardcoded defaults (AveMujicaAPI `gpt-5.5`).

API keys (`OPENAI_API_KEY`, `EXA_API_KEY`) must come from `.env` or the environment only — they are never read from config files. This separation keeps secrets out of version-controlled config files.

Invalid config files print a warning instead of crashing.

## Skill Simplification Notes

Skills should follow the Agent Skills convention with `SKILL.md` and frontmatter. Built-in skills store their prompt content in `src/skills/*.txt` files loaded via `include_str!` for easy editing without recompiling. The first Rust version only needs:

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

## Testing Notes

The Rust version starts with unit tests for diff logic (unified patch generation and terminal diff rendering). Tool and integration tests can be added as the harness matures.
