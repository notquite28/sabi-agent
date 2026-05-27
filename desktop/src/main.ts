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

app.innerHTML = `
  <section class="shell">
    <header>
      <p class="eyebrow">Sabi Agent Desktop</p>
      <h1>Agent sessions</h1>
      <p class="muted">Minimal Tauri shell over the Rust engine boundary.</p>
    </header>
    <section class="card">
      <div class="row">
        <strong>Backend</strong>
        <span id="health" class="pill">checking…</span>
      </div>
      <div class="row">
        <strong>Sessions</strong>
        <button id="refresh" type="button">Refresh</button>
      </div>
      <ul id="sessions" class="sessions"></ul>
    </section>
  </section>
`;

const healthEl = document.querySelector<HTMLSpanElement>("#health");
const sessionsEl = document.querySelector<HTMLUListElement>("#sessions");
const refreshButton = document.querySelector<HTMLButtonElement>("#refresh");

async function loadHealth(): Promise<void> {
  if (!healthEl) return;
  try {
    const health = await invoke<string>("health");
    healthEl.textContent = health;
    healthEl.dataset.ok = "true";
  } catch (error) {
    healthEl.textContent = String(error);
    healthEl.dataset.ok = "false";
  }
}

async function loadSessions(): Promise<void> {
  if (!sessionsEl) return;
  sessionsEl.innerHTML = `<li class="muted">Loading sessions…</li>`;
  try {
    const sessions = await invoke<DesktopSessionInfo[]>("list_sessions");
    if (sessions.length === 0) {
      sessionsEl.innerHTML = `<li class="muted">No saved sessions for this working directory.</li>`;
      return;
    }
    sessionsEl.replaceChildren(
      ...sessions.map((session) => {
        const item = document.createElement("li");
        item.className = "session";
        item.innerHTML = `
          <div>
            <strong>${session.id}</strong>
            <p>${session.path}</p>
          </div>
          <span>${session.message_count} messages</span>
        `;
        return item;
      }),
    );
  } catch (error) {
    sessionsEl.innerHTML = `<li class="error">${String(error)}</li>`;
  }
}

refreshButton?.addEventListener("click", () => {
  void loadSessions();
});

void loadHealth();
void loadSessions();
