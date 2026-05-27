import { invoke } from "@tauri-apps/api/core";
import { MetalFx } from "metal-fx";
import { FormEvent, KeyboardEvent, useEffect, useMemo, useState } from "react";
import { createRoot } from "react-dom/client";
import "./styles.css";

type DesktopSessionInfo = {
  id: string;
  path: string;
  cwd: string;
  message_count: number;
  created_at: string;
  modified_at: string;
};

type HealthState = {
  ok: boolean;
  label: string;
};

function basename(path: string): string {
  const normalized = path.replace(/[/\\]+$/, "");
  const index = Math.max(normalized.lastIndexOf("/"), normalized.lastIndexOf("\\"));
  return index >= 0 ? normalized.slice(index + 1) : normalized;
}

function App() {
  const [workspace, setWorkspace] = useState("");
  const [workspaceDraft, setWorkspaceDraft] = useState("");
  const [sessions, setSessions] = useState<DesktopSessionInfo[]>([]);
  const [sessionsMessage, setSessionsMessage] = useState("Loading sessions…");
  const [health, setHealth] = useState<HealthState>({ ok: false, label: "backend checking…" });

  const workspaceName = useMemo(() => (workspace ? basename(workspace) : "No workspace"), [workspace]);

  async function loadHealth(): Promise<void> {
    try {
      const result = await invoke<string>("health");
      setHealth({ ok: true, label: `backend ${result}` });
    } catch (error) {
      setHealth({ ok: false, label: String(error) });
    }
  }

  async function loadSessions(cwd: string): Promise<void> {
    setSessionsMessage("Loading sessions…");
    try {
      const result = await invoke<DesktopSessionInfo[]>("list_sessions", { cwd: cwd || null });
      setSessions(result);
      setSessionsMessage(result.length === 0 ? "No saved sessions for this workspace." : "");
    } catch (error) {
      setSessions([]);
      setSessionsMessage(String(error));
    }
  }

  useEffect(() => {
    void loadHealth();
    void invoke<string>("current_workspace")
      .then((cwd) => {
        setWorkspace(cwd);
        setWorkspaceDraft(cwd);
        return loadSessions(cwd);
      })
      .catch((error) => {
        setWorkspace("");
        setWorkspaceDraft("");
        setSessions([]);
        setSessionsMessage(String(error));
      });
  }, []);

  function applyWorkspace(): void {
    const nextWorkspace = workspaceDraft.trim();
    if (!nextWorkspace) return;
    setWorkspace(nextWorkspace);
    setSessions([]);
    void loadSessions(nextWorkspace);
  }

  function onWorkspaceKeyDown(event: KeyboardEvent<HTMLInputElement>): void {
    if (event.key === "Enter") {
      event.preventDefault();
      applyWorkspace();
    }
  }

  function preventPromptSubmit(event: FormEvent<HTMLFormElement>): void {
    event.preventDefault();
  }

  const sessionsContent = sessions.length > 0
    ? sessions.map((session) => (
        <li className="session" key={session.id}>
          <button type="button" title={session.path}>
            <span>{session.id.slice(0, 8)}</span>
            <small>{session.message_count} messages</small>
          </button>
        </li>
      ))
    : <li className={sessionsMessage.startsWith("agent error") || sessionsMessage.startsWith("failed") ? "error" : "muted"}>{sessionsMessage}</li>;

  return (
    <div className="app-shell">
      <aside className="sidebar" aria-label="Sabi desktop controls">
        <div className="brand-row">
          <div className="brand-mark" aria-hidden="true">S</div>
          <div>
            <strong>Sabi Agent</strong>
            <p className="health" data-ok={health.ok}>{health.label}</p>
          </div>
        </div>

        <section className="sidebar-section" aria-labelledby="workspace-heading">
          <p id="workspace-heading" className="section-label">Workspace</p>
          <div className="workspace-name" title={workspace}>{workspaceName}</div>
          <label className="workspace-input-label" htmlFor="workspace-input">Path</label>
          <input
            id="workspace-input"
            className="workspace-input"
            type="text"
            spellCheck={false}
            autoComplete="off"
            value={workspaceDraft}
            onChange={(event) => setWorkspaceDraft(event.currentTarget.value)}
            onKeyDown={onWorkspaceKeyDown}
          />
          <div className="workspace-actions">
            <MetalFx variant="button" preset="silver" theme="dark" strength={0.42} disableGlow>
              <button className="secondary-button" type="button" onClick={applyWorkspace}>Use Path</button>
            </MetalFx>
            <button className="secondary-button" type="button" onClick={() => void loadSessions(workspace)}>Refresh</button>
          </div>
        </section>

        <section className="sidebar-section sessions-block" aria-labelledby="sessions-heading">
          <p id="sessions-heading" className="section-label">Sessions</p>
          <ul className="sessions">{sessionsContent}</ul>
        </section>
      </aside>

      <main className="agent-canvas" aria-labelledby="canvas-title">
        <section className="center-card">
          <p className="crumb">Home › Local</p>
          <h1 id="canvas-title">Agent sessions</h1>
          <p className="lede">Browse saved Sabi sessions for the selected workspace. Chat execution will be added when the desktop backend supports prompt turns.</p>
          <form className="composer-shell" aria-label="Prompt composer placeholder" onSubmit={preventPromptSubmit}>
            <textarea disabled rows={3} placeholder="Prompt input is disabled until desktop prompt execution is wired" />
            <div className="composer-footer">
              <span>Workspace-scoped sessions only</span>
              <MetalFx variant="circle" preset="chromatic" theme="dark" strength={0.5}>
                <button type="submit" disabled aria-label="Send prompt">➜</button>
              </MetalFx>
            </div>
          </form>
        </section>
      </main>
    </div>
  );
}

const root = document.querySelector<HTMLElement>("#app");

if (!root) {
  throw new Error("#app root not found");
}

createRoot(root).render(<App />);
