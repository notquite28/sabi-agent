import { invoke } from "@tauri-apps/api/core";
import "./styles.css";

type DesktopSessionInfo = {
  id: string;
  path: string;
  cwd: string;
  message_count: number;
  created_at: string;
  modified_at: string;
};

type DesktopSkillInfo = {
  name: string;
  description: string;
  file_path: string;
};

type DesktopFileSuggestion = {
  path: string;
  name: string;
  is_dir: boolean;
};

type Suggestion = {
  kind: "file" | "command" | "skill";
  value: string;
  label: string;
  detail: string;
};

type CompletionToken = {
  trigger: "@" | "/";
  start: number;
  end: number;
  query: string;
};

const app = document.querySelector<HTMLElement>("#app");

if (!app) {
  throw new Error("#app root not found");
}

const slashCommands: Suggestion[] = [
  { kind: "command", value: "/help", label: "/help", detail: "Show CLI slash commands" },
  { kind: "command", value: "/clear", label: "/clear", detail: "Clear in-memory conversation" },
  { kind: "command", value: "/new", label: "/new", detail: "Start a fresh session" },
  { kind: "command", value: "/session", label: "/session", detail: "Show session metadata" },
  { kind: "command", value: "/reload", label: "/reload", detail: "Reload previous session" },
  { kind: "command", value: "/fiwb", label: "/fiwb", detail: "Toggle approval bypass" },
  { kind: "command", value: "/yolo", label: "/yolo", detail: "Alias for /fiwb" },
  { kind: "command", value: "/quit", label: "/quit", detail: "Exit interactive CLI" },
];

const state: {
  workspace: string;
  sessions: DesktopSessionInfo[];
  skills: DesktopSkillInfo[];
  activeToken: CompletionToken | null;
  activeSuggestions: Suggestion[];
  selectedSuggestion: number;
} = {
  workspace: "",
  sessions: [],
  skills: [],
  activeToken: null,
  activeSuggestions: [],
  selectedSuggestion: 0,
};

app.innerHTML = `
  <div class="app-shell">
    <aside class="sidebar" aria-label="Sabi desktop controls">
      <div class="brand-row">
        <div class="brand-mark" aria-hidden="true">S</div>
        <div>
          <strong>Sabi Agent</strong>
          <p id="health" class="health">backend checking…</p>
        </div>
      </div>

      <section class="sidebar-section" aria-labelledby="workspace-heading">
        <p id="workspace-heading" class="section-label">Workspace</p>
        <div id="workspace-name" class="workspace-name">Loading…</div>
        <label class="workspace-input-label" for="workspace-input">Path</label>
        <input id="workspace-input" class="workspace-input" type="text" spellcheck="false" autocomplete="off" />
        <div class="workspace-actions">
          <button id="apply-workspace" class="secondary-button" type="button">Use Path</button>
          <button id="refresh" class="secondary-button" type="button">Refresh</button>
        </div>
      </section>

      <section class="sidebar-section sessions-block" aria-labelledby="sessions-heading">
        <p id="sessions-heading" class="section-label">Sessions</p>
        <ul id="sessions" class="sessions"></ul>
      </section>
    </aside>

    <main class="agent-canvas" aria-labelledby="canvas-title">
      <section class="agent-card">
        <p class="crumb">Home › Local</p>
        <h1 id="canvas-title">Sabi Agent</h1>
        <p class="lede">Draft prompts with <code>@file</code> and <code>/skill:name</code> completions. Prompt execution is the next backend slice.</p>
        <form id="composer" class="composer" aria-label="Agent prompt composer">
          <textarea id="prompt" rows="5" placeholder="Ask Sabi… Use @ for files, / for commands and skills"></textarea>
          <div id="suggestions" class="suggestions" hidden></div>
          <div class="composer-footer">
            <div class="composer-hints">
              <span><kbd>@</kbd> files</span>
              <span><kbd>/</kbd> commands + skills</span>
              <span><kbd>Tab</kbd> complete</span>
            </div>
            <button id="send" class="send-button" type="submit" disabled title="Prompt execution is not wired yet">Send</button>
          </div>
        </form>
      </section>
    </main>
  </div>
`;

const healthEl = document.querySelector<HTMLParagraphElement>("#health");
const sessionsEl = document.querySelector<HTMLUListElement>("#sessions");
const refreshButton = document.querySelector<HTMLButtonElement>("#refresh");
const workspaceInput = document.querySelector<HTMLInputElement>("#workspace-input");
const workspaceName = document.querySelector<HTMLDivElement>("#workspace-name");
const applyWorkspaceButton = document.querySelector<HTMLButtonElement>("#apply-workspace");
const composer = document.querySelector<HTMLFormElement>("#composer");
const promptInput = document.querySelector<HTMLTextAreaElement>("#prompt");
const suggestionsEl = document.querySelector<HTMLDivElement>("#suggestions");

function basename(path: string): string {
  const normalized = path.replace(/[/\\]+$/, "");
  const index = Math.max(normalized.lastIndexOf("/"), normalized.lastIndexOf("\\"));
  return index >= 0 ? normalized.slice(index + 1) : normalized;
}

function renderWorkspace(): void {
  if (workspaceInput) {
    workspaceInput.value = state.workspace;
  }
  if (workspaceName) {
    workspaceName.textContent = state.workspace ? basename(state.workspace) : "No workspace";
    workspaceName.title = state.workspace;
  }
}

function renderSessions(): void {
  if (!sessionsEl) return;

  if (!state.workspace) {
    sessionsEl.innerHTML = `<li class="muted">Choose a workspace to load sessions.</li>`;
    return;
  }

  if (state.sessions.length === 0) {
    sessionsEl.innerHTML = `<li class="muted">No saved sessions for this workspace.</li>`;
    return;
  }

  sessionsEl.replaceChildren(
    ...state.sessions.map((session) => {
      const item = document.createElement("li");
      item.className = "session";
      item.innerHTML = `
        <button type="button" title="${escapeHtml(session.path)}">
          <span>${escapeHtml(session.id.slice(0, 8))}</span>
          <small>${session.message_count} messages</small>
        </button>
      `;
      return item;
    }),
  );
}

function renderSuggestions(): void {
  if (!suggestionsEl) return;

  if (!state.activeToken || state.activeSuggestions.length === 0) {
    suggestionsEl.hidden = true;
    suggestionsEl.replaceChildren();
    return;
  }

  suggestionsEl.hidden = false;
  suggestionsEl.replaceChildren(
    ...state.activeSuggestions.map((suggestion, index) => {
      const button = document.createElement("button");
      button.type = "button";
      button.className = "suggestion";
      button.dataset.selected = String(index === state.selectedSuggestion);
      button.innerHTML = `
        <span class="suggestion-kind">${suggestion.kind}</span>
        <span class="suggestion-main">${escapeHtml(suggestion.label)}</span>
        <span class="suggestion-detail">${escapeHtml(suggestion.detail)}</span>
      `;
      button.addEventListener("mousedown", (event) => {
        event.preventDefault();
        acceptSuggestion(index);
      });
      return button;
    }),
  );
}

async function loadHealth(): Promise<void> {
  if (!healthEl) return;
  try {
    const health = await invoke<string>("health");
    healthEl.textContent = `backend ${health}`;
    healthEl.dataset.ok = "true";
  } catch (error) {
    healthEl.textContent = String(error);
    healthEl.dataset.ok = "false";
  }
}

async function loadWorkspace(): Promise<void> {
  try {
    state.workspace = await invoke<string>("current_workspace");
    renderWorkspace();
  } catch (error) {
    if (workspaceName) {
      workspaceName.textContent = "Workspace unavailable";
      workspaceName.title = String(error);
    }
  }
}

async function loadSkills(): Promise<void> {
  try {
    state.skills = await invoke<DesktopSkillInfo[]>("list_skills", { cwd: state.workspace || null });
  } catch {
    state.skills = [];
  }
}

async function loadSessions(): Promise<void> {
  if (!sessionsEl) return;
  sessionsEl.innerHTML = `<li class="muted">Loading sessions…</li>`;
  try {
    state.sessions = await invoke<DesktopSessionInfo[]>("list_sessions", { cwd: state.workspace || null });
    renderSessions();
  } catch (error) {
    sessionsEl.innerHTML = `<li class="error">${escapeHtml(String(error))}</li>`;
  }
}

async function loadFileSuggestions(query: string): Promise<Suggestion[]> {
  try {
    const files = await invoke<DesktopFileSuggestion[]>("list_workspace_files", {
      cwd: state.workspace || null,
      query,
    });
    return files.map((file) => ({
      kind: "file",
      value: `@${file.path}`,
      label: file.path,
      detail: file.is_dir ? "directory" : "file",
    }));
  } catch {
    return [];
  }
}

function skillSuggestions(query: string): Suggestion[] {
  const normalized = query.toLowerCase().replace(/^skill:?/, "");
  const skillItems = state.skills.map((skill) => ({
    kind: "skill" as const,
    value: `/skill:${skill.name}`,
    label: `/skill:${skill.name}`,
    detail: skill.description,
  }));
  return [...slashCommands, ...skillItems].filter((suggestion) =>
    suggestion.label.toLowerCase().includes(normalized),
  );
}

async function updateCompletions(): Promise<void> {
  if (!promptInput) return;
  const token = completionTokenAt(promptInput.value, promptInput.selectionStart);
  state.activeToken = token;
  state.selectedSuggestion = 0;

  if (!token) {
    state.activeSuggestions = [];
    renderSuggestions();
    return;
  }

  state.activeSuggestions = token.trigger === "@"
    ? await loadFileSuggestions(token.query)
    : skillSuggestions(token.query);
  renderSuggestions();
}

function completionTokenAt(value: string, cursor: number): CompletionToken | null {
  let start = cursor;
  while (start > 0 && !/\s/.test(value[start - 1])) {
    start -= 1;
  }

  const token = value.slice(start, cursor);
  const trigger = token[0];
  if (trigger !== "@" && trigger !== "/") {
    return null;
  }
  return {
    trigger,
    start,
    end: cursor,
    query: token.slice(1),
  };
}

function acceptSuggestion(index = state.selectedSuggestion): void {
  if (!promptInput || !state.activeToken) return;
  const suggestion = state.activeSuggestions[index];
  if (!suggestion) return;

  const before = promptInput.value.slice(0, state.activeToken.start);
  const after = promptInput.value.slice(state.activeToken.end);
  const insertion = `${suggestion.value} `;
  promptInput.value = `${before}${insertion}${after}`;
  const nextCursor = before.length + insertion.length;
  promptInput.setSelectionRange(nextCursor, nextCursor);
  promptInput.focus();

  state.activeToken = null;
  state.activeSuggestions = [];
  state.selectedSuggestion = 0;
  renderSuggestions();
}

function applyWorkspace(): void {
  const nextWorkspace = workspaceInput?.value.trim();
  if (!nextWorkspace) return;
  state.workspace = nextWorkspace;
  state.sessions = [];
  state.skills = [];
  renderWorkspace();
  void loadSessions();
  void loadSkills();
}

function escapeHtml(value: string): string {
  return value.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
}

refreshButton?.addEventListener("click", () => {
  void loadSessions();
  void loadSkills();
});

applyWorkspaceButton?.addEventListener("click", applyWorkspace);

workspaceInput?.addEventListener("keydown", (event) => {
  if (event.key === "Enter") {
    event.preventDefault();
    applyWorkspace();
  }
});

promptInput?.addEventListener("input", () => {
  void updateCompletions();
});

promptInput?.addEventListener("click", () => {
  void updateCompletions();
});

promptInput?.addEventListener("keydown", (event) => {
  if (!state.activeToken || state.activeSuggestions.length === 0) return;

  if (event.key === "ArrowDown") {
    event.preventDefault();
    state.selectedSuggestion = (state.selectedSuggestion + 1) % state.activeSuggestions.length;
    renderSuggestions();
  } else if (event.key === "ArrowUp") {
    event.preventDefault();
    state.selectedSuggestion = (state.selectedSuggestion - 1 + state.activeSuggestions.length) % state.activeSuggestions.length;
    renderSuggestions();
  } else if (event.key === "Tab" || event.key === "Enter") {
    event.preventDefault();
    acceptSuggestion();
  } else if (event.key === "Escape") {
    event.preventDefault();
    state.activeToken = null;
    state.activeSuggestions = [];
    renderSuggestions();
  }
});

composer?.addEventListener("submit", (event) => {
  event.preventDefault();
});

await loadWorkspace();
void loadHealth();
void loadSessions();
void loadSkills();
