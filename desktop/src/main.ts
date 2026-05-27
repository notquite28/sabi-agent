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

const app = document.querySelector<HTMLElement>("#app");

if (!app) {
  throw new Error("#app root not found");
}

const state: {
  workspace: string;
  sessions: DesktopSessionInfo[];
} = {
  workspace: "",
  sessions: [],
};

app.innerHTML = `
  <div class="app-shell">
    <aside class="sidebar" aria-label="Primary navigation">
      <div class="sidebar-topbar">
        <button class="icon-button" type="button" aria-label="Toggle sidebar">▱</button>
        <button class="icon-button" type="button" aria-label="Search">⌕</button>
      </div>

      <nav class="nav-stack" aria-label="Agent navigation">
        <button class="nav-item active" type="button"><span>◇</span> New Agent <kbd>Ctrl N</kbd></button>
        <button class="nav-item" type="button"><span>□</span> Sessions</button>
        <button class="nav-item" type="button"><span>✦</span> Skills</button>
      </nav>

      <section class="workspace-block" aria-labelledby="workspace-heading">
        <p id="workspace-heading" class="section-label">Workspace</p>
        <div id="workspace-name" class="workspace-name">Loading…</div>
        <label class="workspace-input-label" for="workspace-input">Path</label>
        <input id="workspace-input" class="workspace-input" type="text" spellcheck="false" autocomplete="off" />
        <div class="workspace-actions">
          <button id="apply-workspace" class="secondary-button" type="button">Use Path</button>
          <button id="refresh" class="secondary-button" type="button">Refresh</button>
        </div>
      </section>

      <section class="sessions-block" aria-labelledby="sessions-heading">
        <p id="sessions-heading" class="section-label">Recent sessions</p>
        <ul id="sessions" class="sessions"></ul>
      </section>

      <footer class="sidebar-footer">
        <div class="avatar" aria-hidden="true">SA</div>
        <div>
          <strong>Sabi Agent</strong>
          <p id="health" class="health">backend checking…</p>
        </div>
      </footer>
    </aside>

    <main class="agent-canvas">
      <div class="topbar">
        <div class="menu-items" aria-label="Application menu">
          <span>File</span>
          <span>Edit</span>
          <span>View</span>
          <span>Help</span>
        </div>
        <div class="window-status">Desktop Shell · Local</div>
      </div>

      <section class="composer-wrap" aria-labelledby="composer-heading">
        <div class="crumb">Home › Local</div>
        <h1 id="composer-heading">What should Sabi build?</h1>
        <form id="composer" class="composer">
          <textarea id="prompt" placeholder="Plan, build, / for commands, @ for context" rows="3"></textarea>
          <div class="composer-footer">
            <div class="composer-tools">
              <button class="round-button" type="button" aria-label="Attach context">＋</button>
              <button class="model-button" type="button">Agent · Local</button>
            </div>
            <button id="send" class="send-button" type="submit" disabled title="Prompt execution is the next backend slice">➜</button>
          </div>
        </form>
        <p class="composer-note">Prompt execution, event streaming, and approval cards are next. This shell is wired for workspace-aware session browsing.</p>
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
        <button type="button" title="${session.path}">
          <span>${session.id.slice(0, 8)}</span>
          <small>${session.message_count} messages</small>
        </button>
      `;
      return item;
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

async function loadSessions(): Promise<void> {
  if (!sessionsEl) return;
  sessionsEl.innerHTML = `<li class="muted">Loading sessions…</li>`;
  try {
    state.sessions = await invoke<DesktopSessionInfo[]>("list_sessions", { cwd: state.workspace || null });
    renderSessions();
  } catch (error) {
    sessionsEl.innerHTML = `<li class="error">${String(error)}</li>`;
  }
}

refreshButton?.addEventListener("click", () => {
  void loadSessions();
});

applyWorkspaceButton?.addEventListener("click", () => {
  const nextWorkspace = workspaceInput?.value.trim();
  if (!nextWorkspace) return;
  state.workspace = nextWorkspace;
  state.sessions = [];
  renderWorkspace();
  void loadSessions();
});

workspaceInput?.addEventListener("keydown", (event) => {
  if (event.key === "Enter") {
    event.preventDefault();
    applyWorkspaceButton?.click();
  }
});

composer?.addEventListener("submit", (event) => {
  event.preventDefault();
});

await loadWorkspace();
void loadHealth();
void loadSessions();
