import { useEffect, useState, type FormEvent } from "react";
import { testAssistantConnection } from "../services/ai";
import { memoryService } from "../services/memory";
import { clearApiKey, saveApiKey, saveProviderSettings, clearAllMemory, getActionLog, clearActionLog } from "../services/tauri";
import { useEdith } from "../state/EdithContext";
import type { AiProviderId } from "../types";
import { toUserError } from "../utils";

const providers: { id: AiProviderId; label: string }[] = [
  { id: "openai", label: "OpenAI" },
  { id: "gemini", label: "Google Gemini" },
  { id: "claude", label: "Anthropic Claude" },
];

const defaultModels: Record<AiProviderId, string> = {
  openai: "gpt-4o-mini",
  gemini: "gemini-3.1-flash-lite",
  claude: "claude-sonnet-4-20250514",
};

export function SettingsView() {
  const { settings, refreshSettings, logActivity, memoryCount, setMemoryCount, refreshMemoryCount } =
    useEdith();
  const [provider, setProvider] = useState<AiProviderId>("openai");
  const [model, setModel] = useState(defaultModels.openai);
  const [apiKey, setApiKey] = useState("");
  const [showKey, setShowKey] = useState(false);
  const [status, setStatus] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [connectionTested, setConnectionTested] = useState(false);
  const [connectionSuccessful, setConnectionSuccessful] = useState(false);
  const [memoryKey, setMemoryKey] = useState("");
  const [memoryValue, setMemoryValue] = useState("");
  const [actionLog, setActionLog] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<"ai" | "storage" | "security">("ai");

  useEffect(() => {
    if (!settings) {
      return;
    }
    setProvider(settings.provider);
    setModel(settings.model || defaultModels[settings.provider]);
    setConnectionTested(false);
    setConnectionSuccessful(false);
  }, [settings]);

  async function onConnect(event: FormEvent) {
    event.preventDefault();
    setBusy(true);
    setError(null);
    setStatus(null);
    try {
      const trimmedKey = apiKey.trim();
      if (!trimmedKey && !settings?.hasApiKey) {
        throw new Error("Voer een API key in voordat je verbindt.");
      }
      await saveProviderSettings(provider, model.trim() || defaultModels[provider]);
      if (trimmedKey) {
        await saveApiKey(trimmedKey);
        setApiKey("");
      }
      await refreshSettings();
      logActivity("Provider-instellingen opgeslagen");
      setStatus("Provider-instellingen opgeslagen. Klik 'Test Connection' om de verbinding te verifiëren.");
    } catch (caught) {
      setError(toUserError(caught));
    } finally {
      setBusy(false);
    }
  }

  async function onTestConnection() {
    setBusy(true);
    setError(null);
    setStatus(null);
    setConnectionTested(false);
    setConnectionSuccessful(false);
    try {
      await saveProviderSettings(provider, model.trim() || defaultModels[provider]);
      const trimmedKey = apiKey.trim();
      if (trimmedKey) {
        await saveApiKey(trimmedKey);
        setApiKey("");
      } else if (!settings?.hasApiKey) {
        throw new Error("Verbind eerst met een API key.");
      }
      await refreshSettings();
      setStatus("Verbinding testen...");
      await testAssistantConnection();
      setConnectionTested(true);
      setConnectionSuccessful(true);
      logActivity("Provider-verbinding getest");
      setStatus("Verbinding geslaagd. API key is geldig en model is beschikbaar.");
    } catch (caught) {
      setConnectionTested(true);
      setConnectionSuccessful(false);
      setError(toUserError(caught));
      setStatus("Verbinding mislukt.");
    } finally {
      setBusy(false);
    }
  }

  async function onDisconnect() {
    setBusy(true);
    setError(null);
    setStatus(null);
    try {
      await clearApiKey();
      setApiKey("");
      await refreshSettings();
      logActivity("Provider ontkoppeld");
      setStatus("API key gewist uit runtime-geheugen en lokale opslag.");
    } catch (caught) {
      setError(toUserError(caught));
    } finally {
      setBusy(false);
    }
  }

  async function onClearMemory() {
    setBusy(true);
    setError(null);
    setStatus(null);
    try {
      await clearAllMemory();
      setMemoryCount(0);
      logActivity("Memory gewist");
      setStatus("Alle memory items gewist.");
    } catch (caught) {
      setError(toUserError(caught));
    } finally {
      setBusy(false);
    }
  }

  async function onViewActionLog() {
    setBusy(true);
    setError(null);
    setActionLog(null);
    try {
      const log = await getActionLog(20);
      const logText = log.map(([timestamp, action, tool, result]) =>
        `[${new Date(timestamp * 1000).toLocaleTimeString()}] ${action} (${tool}): ${result}`
      ).join("\n");
      setActionLog(logText || "Geen recente activiteit.");
    } catch (caught) {
      setError(toUserError(caught));
    } finally {
      setBusy(false);
    }
  }

  async function onClearActionLog() {
    setBusy(true);
    setError(null);
    setActionLog(null);
    try {
      await clearActionLog();
      logActivity("Action log gewist");
      setStatus("Action log gewist.");
    } catch (caught) {
      setError(toUserError(caught));
    } finally {
      setBusy(false);
    }
  }

  function saveMemory(event: FormEvent) {
    event.preventDefault();
    if (!memoryKey.trim() || !memoryValue.trim()) {
      return;
    }
    memoryService.saveMemory(memoryKey, memoryValue);
    setMemoryKey("");
    setMemoryValue("");
    refreshMemoryCount();
  }

  return (
    <section className="view">
      <header className="page-header">
        <h2>Settings</h2>
        <p className="subtitle">
          API keys blijven in Rust-runtimegeheugen en lokale opslag. Ze staan niet in de frontend-bundle.
        </p>
      </header>

      <div className="settings-tabs">
        <button
          type="button"
          className={activeTab === "ai" ? "active" : ""}
          onClick={() => setActiveTab("ai")}
        >
          AI Provider
        </button>
        <button
          type="button"
          className={activeTab === "storage" ? "active" : ""}
          onClick={() => setActiveTab("storage")}
        >
          Storage
        </button>
        <button
          type="button"
          className={activeTab === "security" ? "active" : ""}
          onClick={() => setActiveTab("security")}
        >
          Security
        </button>
      </div>

      {activeTab === "ai" && (
        <form className="settings-form" onSubmit={onConnect}>
        <label htmlFor="provider">AI provider</label>
        <select
          id="provider"
          value={provider}
          onChange={(event) => {
            const next = event.target.value;
            const match = providers.find((item) => item.id === next);
            if (!match) {
              return;
            }
            setProvider(match.id);
            setModel(defaultModels[match.id]);
          }}
        >
          {providers.map((item) => (
            <option key={item.id} value={item.id}>
              {item.label}
            </option>
          ))}
        </select>

        <label htmlFor="model">Model</label>
        <input
          id="model"
          value={model}
          onChange={(event) => setModel(event.target.value)}
        />

        <label htmlFor="api-key">API key</label>
        <div className="key-row">
          <input
            id="api-key"
            type={showKey ? "text" : "password"}
            value={apiKey}
            autoComplete="off"
            spellCheck={false}
            placeholder={
              settings?.maskedApiKey
                ? "Nieuwe key invoeren om te vervangen"
                : "Plak je API key"
            }
            onChange={(event) => setApiKey(event.target.value)}
          />
          <button
            type="button"
            className="ghost"
            onClick={() => setShowKey((value) => !value)}
          >
            {showKey ? "Hide" : "Show"}
          </button>
        </div>

        <p className="muted">
          Opgeslagen key:{" "}
          {settings?.maskedApiKey ?? "geen key in geheugen"}
        </p>

        {connectionTested && (
          <p className={connectionSuccessful ? "success-text" : "error-text"}>
            {connectionSuccessful
              ? "✓ Verbinding getest en werkend"
              : "✗ Verbindingstest mislukt"}
          </p>
        )}

        <div className="button-row">
          <button type="submit" disabled={busy}>
            Connect
          </button>
          <button
            type="button"
            className="ghost"
            onClick={() => void onTestConnection()}
            disabled={busy}
          >
            Test Connection
          </button>
          <button
            type="button"
            className="ghost"
            onClick={() => void onDisconnect()}
            disabled={busy || !settings?.hasApiKey}
          >
            Disconnect
          </button>
        </div>
      </form>
      )}

      {activeTab === "storage" && (
        <div className="settings-form">
          <article className="panel">
            <h3>Memory</h3>
            <p className="muted">
              Persistente lokale memory ({memoryCount} items). SQLite backend actief.
            </p>
            <form className="inline-form" onSubmit={saveMemory}>
              <input
                value={memoryKey}
                onChange={(event) => setMemoryKey(event.target.value)}
                placeholder="Sleutel"
                aria-label="Memory sleutel"
              />
              <input
                value={memoryValue}
                onChange={(event) => setMemoryValue(event.target.value)}
                placeholder="Waarde"
                aria-label="Memory waarde"
              />
              <button type="submit">Bewaar</button>
            </form>
            <button
              type="button"
              className="ghost"
              onClick={() => void onClearMemory()}
              disabled={busy}
            >
              Clear All Memory
            </button>
          </article>

          <article className="panel">
            <h3>Conversations</h3>
            <p className="muted">
              Conversaties worden lokaal opgeslagen en blijven beschikbaar na herstart.
            </p>
          </article>

          <article className="panel">
            <h3>Action Log</h3>
            <p className="muted">
              Log van alle uitgevoerde desktop acties en tool calls.
            </p>
            <div className="button-row">
              <button
                type="button"
                className="ghost"
                onClick={() => void onViewActionLog()}
                disabled={busy}
              >
                View Recent Actions
              </button>
              <button
                type="button"
                className="ghost"
                onClick={() => void onClearActionLog()}
                disabled={busy}
              >
                Clear Action Log
              </button>
            </div>
            {actionLog && (
              <pre className="action-log-display">{actionLog}</pre>
            )}
          </article>
        </div>
      )}

      {activeTab === "security" && (
        <div className="settings-form">
          <article className="panel">
            <h3>Security Information</h3>
            <ul className="plain-list">
              <li>API keys worden opgeslagen in lokale SQLite database, niet in plaintext.</li>
              <li>Keys gaan via Tauri commands naar Rust-state, niet naar localStorage.</li>
              <li>De UI krijgt na opslaan alleen een gemaskeerde weergave.</li>
              <li>Desktop tools vereisen permissies en bevestiging voor destructieve acties.</li>
              <li>Geen API keys worden verzonden naar externe servers.</li>
              <li>Alle acties worden gelogd voor transparantie.</li>
            </ul>
          </article>

          <article className="panel">
            <h3>Data Management</h3>
            <p className="muted">
              U kunt alle lokale data wissen door E.D.I.T.H. volledig te verwijderen.
            </p>
          </article>
        </div>
      )}

      {status ? <p className="success-text">{status}</p> : null}
      {error ? (
        <p className="error-text" role="alert">
          {error}
        </p>
      ) : null}
    </section>
  );
}
