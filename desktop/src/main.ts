import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import "./styles.css";

type DesktopSessionInfo = {
  id: string;
  title: string | null;
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

type TranscriptItem = {
  kind: "user" | "assistant" | "event" | "error";
  title?: string;
  text: string;
};

type PromptResponse = {
  reply: string;
  events: unknown[];
  session: DesktopSessionInfo;
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
  contextSession: DesktopSessionInfo | null;
  transcript: TranscriptItem[];
  sending: boolean;
} = {
  workspace: "",
  sessions: [],
  skills: [],
  activeToken: null,
  activeSuggestions: [],
  selectedSuggestion: 0,
  contextSession: null,
  transcript: [],
  sending: false,
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
        <div class="project-card">
          <div>
            <div id="workspace-name" class="workspace-name">Loading…</div>
            <div id="workspace-path" class="workspace-path"></div>
          </div>
          <button id="refresh" class="icon-button" type="button" title="Refresh workspace" aria-label="Refresh workspace">↻</button>
        </div>
        <button id="open-project" class="secondary-button full-width" type="button">Open Project</button>
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
        <p class="lede">Ask Sabi to inspect and explain the current project. File edits and shell commands wait for the approval UI.</p>
        <div id="transcript" class="transcript" aria-live="polite"></div>
        <form id="composer" class="composer" aria-label="Agent prompt composer">
          <textarea id="prompt" rows="5" placeholder="Ask Sabi… Use @ for files, / for commands and skills"></textarea>
          <div id="suggestions" class="suggestions" hidden></div>
          <div class="composer-footer">
            <div class="composer-hints">
              <span><kbd>@</kbd> files</span>
              <span><kbd>/</kbd> commands + skills</span>
              <span><kbd>Tab</kbd> complete</span>
            </div>
            <button id="send" class="send-button" type="submit">Send</button>
          </div>
        </form>
      </section>
    </main>
    <div id="session-menu" class="context-menu" hidden>
      <button id="delete-session" type="button">Delete Session</button>
    </div>
  </div>
`;

const healthEl = document.querySelector<HTMLParagraphElement>("#health");
const sessionsEl = document.querySelector<HTMLUListElement>("#sessions");
const refreshButton = document.querySelector<HTMLButtonElement>("#refresh");
const workspaceName = document.querySelector<HTMLDivElement>("#workspace-name");
const workspacePath = document.querySelector<HTMLDivElement>("#workspace-path");
const openProjectButton = document.querySelector<HTMLButtonElement>("#open-project");
const composer = document.querySelector<HTMLFormElement>("#composer");
const promptInput = document.querySelector<HTMLTextAreaElement>("#prompt");
const suggestionsEl = document.querySelector<HTMLDivElement>("#suggestions");
const sessionMenu = document.querySelector<HTMLDivElement>("#session-menu");
const deleteSessionButton = document.querySelector<HTMLButtonElement>("#delete-session");
const transcriptEl = document.querySelector<HTMLDivElement>("#transcript");
const sendButton = document.querySelector<HTMLButtonElement>("#send");

function basename(path: string): string {
  const normalized = path.replace(/[/\\]+$/, "");
  const index = Math.max(normalized.lastIndexOf("/"), normalized.lastIndexOf("\\"));
  return index >= 0 ? normalized.slice(index + 1) : normalized;
}

function renderWorkspace(): void {
  if (workspaceName) {
    workspaceName.textContent = state.workspace ? basename(state.workspace) : "No workspace";
    workspaceName.title = state.workspace;
  }
  if (workspacePath) {
    workspacePath.textContent = state.workspace;
    workspacePath.title = state.workspace;
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
          <span class="session-title">${escapeHtml(sessionTitle(session))}</span>
          <small>${session.message_count} messages · ${escapeHtml(session.id.slice(0, 8))}</small>
        </button>
      `;
      item.addEventListener("contextmenu", (event) => {
        event.preventDefault();
        showSessionMenu(event.clientX, event.clientY, session);
      });
      return item;
    }),
  );
}

function renderTranscript(): void {
  if (!transcriptEl) return;
  if (state.transcript.length === 0) {
    transcriptEl.replaceChildren();
    return;
  }
  transcriptEl.replaceChildren(
    ...state.transcript.map((item) => {
      const row = document.createElement("article");
      row.className = `transcript-item transcript-${item.kind}`;
      const title = item.title || item.kind;
      row.innerHTML = `
        <div class="transcript-title">${escapeHtml(title)}</div>
        <div class="transcript-text">${escapeHtml(item.text)}</div>
      `;
      return row;
    }),
  );
  transcriptEl.scrollTop = transcriptEl.scrollHeight;
}

function renderSendState(): void {
  if (!sendButton || !promptInput) return;
  sendButton.disabled = state.sending || promptInput.value.trim().length === 0;
  sendButton.textContent = state.sending ? "Sending" : "Send";
}

function showSessionMenu(x: number, y: number, session: DesktopSessionInfo): void {
  if (!sessionMenu) return;
  state.contextSession = session;
  sessionMenu.hidden = false;
  sessionMenu.style.left = `${x}px`;
  sessionMenu.style.top = `${y}px`;
}

function hideSessionMenu(): void {
  if (!sessionMenu) return;
  sessionMenu.hidden = true;
  state.contextSession = null;
}

async function deleteContextSession(): Promise<void> {
  const session = state.contextSession;
  if (!session) return;
  hideSessionMenu();
  const confirmed = window.confirm(`Delete session "${sessionTitle(session)}"?`);
  if (!confirmed) return;
  await invoke<boolean>("delete_session", { cwd: state.workspace || null, id: session.id });
  state.sessions = state.sessions.filter((item) => item.id !== session.id);
  renderSessions();
}

function sessionTitle(session: DesktopSessionInfo): string {
  return session.title?.trim() || `Session ${session.id.slice(0, 8)}`;
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
  const selected = suggestionsEl.querySelector<HTMLElement>('[data-selected="true"]');
  selected?.scrollIntoView({ block: "nearest" });
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

async function startSession(): Promise<void> {
  if (!state.workspace) return;
  try {
    const session = await invoke<DesktopSessionInfo>("start_or_resume_session", { cwd: state.workspace });
    const existing = state.sessions.findIndex((item) => item.id === session.id);
    if (existing >= 0) {
      state.sessions[existing] = session;
    } else if (session.message_count > 0) {
      state.sessions.unshift(session);
    }
    renderSessions();
  } catch (error) {
    state.transcript.push({ kind: "error", title: "Session Error", text: String(error) });
    renderTranscript();
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

function setWorkspace(nextWorkspace: string): void {
  state.workspace = nextWorkspace;
  state.sessions = [];
  state.skills = [];
  state.transcript = [];
  renderWorkspace();
  renderTranscript();
  void loadSessions();
  void loadSkills();
  void startSession();
}

async function sendPrompt(): Promise<void> {
  if (!promptInput || state.sending) return;
  const prompt = promptInput.value.trim();
  if (!prompt) return;

  state.sending = true;
  renderSendState();
  promptInput.value = "";
  state.activeToken = null;
  state.activeSuggestions = [];
  renderSuggestions();
  state.transcript.push({ kind: "user", title: "You", text: prompt });
  renderTranscript();

  try {
    const response = await invoke<PromptResponse>("send_prompt", { cwd: state.workspace || null, prompt });
    for (const event of response.events) {
      const item = transcriptItemFromEvent(event);
      if (item) state.transcript.push(item);
    }
    if (response.reply.trim()) {
      state.transcript.push({ kind: "assistant", title: "Sabi", text: response.reply });
    }
    upsertSession(response.session);
    renderSessions();
  } catch (error) {
    state.transcript.push({ kind: "error", title: "Prompt Error", text: String(error) });
  } finally {
    state.sending = false;
    renderSendState();
    renderTranscript();
    void loadSessions();
  }
}

function upsertSession(session: DesktopSessionInfo): void {
  const index = state.sessions.findIndex((item) => item.id === session.id);
  if (index >= 0) {
    state.sessions[index] = session;
  } else {
    state.sessions.unshift(session);
  }
}

function transcriptItemFromEvent(event: unknown): TranscriptItem | null {
  if (!event || typeof event !== "object") return null;
  const entries = Object.entries(event as Record<string, unknown>);
  const [kind, payload] = entries[0] || [];
  if (!kind || !payload || typeof payload !== "object") return null;
  const data = payload as Record<string, unknown>;
  switch (kind) {
    case "AssistantText":
      return null;
    case "ToolStarted":
      return { kind: "event", title: `Tool started: ${String(data.name || "tool")}`, text: JSON.stringify(data.args ?? {}, null, 2) };
    case "ToolFinished":
      return { kind: data.is_error ? "error" : "event", title: `Tool finished: ${String(data.name || "tool")}`, text: String(data.output || "") };
    case "DiffReady":
      return { kind: "event", title: `Diff: ${String(data.path || "file")}`, text: String(data.patch || data.rendered || "") };
    case "FileChanged":
      return { kind: "event", title: "File changed", text: String(data.path || "") };
    case "Error":
      return { kind: "error", title: "Error", text: String(data.message || "") };
    default:
      return null;
  }
}

async function openProject(): Promise<void> {
  const selected = await open({
    directory: true,
    multiple: false,
    title: "Open Project",
    defaultPath: state.workspace || undefined,
  });
  if (typeof selected === "string") {
    setWorkspace(selected);
  }
}

function escapeHtml(value: string): string {
  return value.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
}

refreshButton?.addEventListener("click", () => {
  void loadSessions();
  void loadSkills();
});

openProjectButton?.addEventListener("click", () => {
  void openProject();
});

promptInput?.addEventListener("input", () => {
  renderSendState();
  void updateCompletions();
});

promptInput?.addEventListener("click", () => {
  void updateCompletions();
});

promptInput?.addEventListener("keydown", (event) => {
  const hasSuggestions = state.activeToken && state.activeSuggestions.length > 0;

  if (hasSuggestions) {
    if (event.key === "ArrowDown") {
      event.preventDefault();
      state.selectedSuggestion = (state.selectedSuggestion + 1) % state.activeSuggestions.length;
      renderSuggestions();
      return;
    }
    if (event.key === "ArrowUp") {
      event.preventDefault();
      state.selectedSuggestion = (state.selectedSuggestion - 1 + state.activeSuggestions.length) % state.activeSuggestions.length;
      renderSuggestions();
      return;
    }
    if (event.key === "Tab" || (event.key === "Enter" && !event.shiftKey)) {
      event.preventDefault();
      acceptSuggestion();
      return;
    }
    if (event.key === "Escape") {
      event.preventDefault();
      state.activeToken = null;
      state.activeSuggestions = [];
      renderSuggestions();
      return;
    }
  }

  if (event.key === "Enter" && !event.shiftKey) {
    event.preventDefault();
    void sendPrompt();
  }
});

deleteSessionButton?.addEventListener("click", () => {
  void deleteContextSession();
});

document.addEventListener("click", (event) => {
  if (sessionMenu && !sessionMenu.hidden && !sessionMenu.contains(event.target as Node)) {
    hideSessionMenu();
  }
});

document.addEventListener("keydown", (event) => {
  if (event.key === "Escape") {
    hideSessionMenu();
  }
});

composer?.addEventListener("submit", (event) => {
  event.preventDefault();
  void sendPrompt();
});

await loadWorkspace();
void loadHealth();
void loadSessions();
void loadSkills();
void startSession();
renderSendState();
