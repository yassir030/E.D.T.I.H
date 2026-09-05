import { useEdith } from "../state/EdithContext";
import { formatTime } from "../utils";

export function DashboardView() {
  const {
    setView,
    settings,
    systemStatus,
    tasks,
    messages,
    activities,
    memoryCount,
  } = useEdith();

  const openTasks = tasks.filter((task) => !task.completed).length;
  const providerLabel = settings
    ? `${providerName(settings.provider)} (${settings.model})`
    : "Niet geladen";

  return (
    <section className="view">
      <header className="page-header">
        <h2>Welcome back, Yassir</h2>
        <p className="subtitle">E.D.I.T.H. is ready.</p>
      </header>

      <div className="cards">
        <article className="card">
          <h3>AI Core</h3>
          <p>
            {settings?.hasApiKey
              ? "● Provider geconfigureerd. Chat is beschikbaar."
              : "○ Nog geen API key. Configureer een provider in Settings."}
          </p>
        </article>
        <article className="card">
          <h3>System</h3>
          <p>
            {systemStatus
              ? `${systemStatus.platform} · v${systemStatus.version}`
              : "Systeemstatus wordt geladen…"}
          </p>
        </article>
        <article className="card">
          <h3>Memory</h3>
          <p>
            {systemStatus?.memoryBackend === "sqlite"
              ? "● Local storage active"
              : "○ Runtime memory only"} · {memoryCount}{" "}
            {memoryCount === 1 ? "item" : "items"}
          </p>
        </article>
        <article className="card">
          <h3>Current provider</h3>
          <p>{providerLabel}</p>
        </article>
        <article className="card">
          <h3>Filesystem</h3>
          <p>
            {systemStatus?.filesystemReady
              ? "● Available"
              : "○ Not available"}
          </p>
        </article>
        <article className="card">
          <h3>Desktop Control</h3>
          <p>
            {systemStatus?.desktopControlReady
              ? "● Available"
              : "○ Not available (platform limitation)"}
          </p>
        </article>
      </div>

      <div className="split">
        <section className="panel">
          <h3>Quick actions</h3>
          <div className="action-grid">
            <button type="button" onClick={() => setView("assistant")}>
              Open Assistant
            </button>
            <button type="button" onClick={() => setView("tasks")}>
              New Task
            </button>
            <button type="button" onClick={() => setView("files")}>
              Open Files
            </button>
            <button type="button" onClick={() => setView("coding")}>
              Open Coding
            </button>
            <button type="button" onClick={() => setView("settings")}>
              Settings
            </button>
          </div>
        </section>

        <section className="panel">
          <h3>Recent activity</h3>
          {activities.length === 0 && messages.length === 0 && openTasks === 0 ? (
            <p className="muted">Nog geen activiteit in deze sessie.</p>
          ) : (
            <ul className="activity-list">
              {activities.slice(0, 6).map((item) => (
                <li key={item.id}>
                  <span>{item.label}</span>
                  <time dateTime={new Date(item.timestamp).toISOString()}>
                    {formatTime(item.timestamp)}
                  </time>
                </li>
              ))}
              {openTasks > 0 ? (
                <li>
                  <span>
                    {openTasks} openstaande {openTasks === 1 ? "taak" : "taken"}
                  </span>
                </li>
              ) : null}
            </ul>
          )}
        </section>
      </div>
    </section>
  );
}

function providerName(id: string): string {
  if (id === "gemini") return "Google Gemini";
  if (id === "claude") return "Anthropic Claude";
  return "OpenAI";
}
