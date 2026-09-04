import { useEffect, useState, type FormEvent } from "react";
import { memoryService } from "../services/memory";
import { clearApiKey, saveApiKey, saveProviderSettings } from "../services/tauri";
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
  gemini: "gemini-2.0-flash",
  claude: "claude-sonnet-4-20250514",
};

export function SettingsView() {
  const { settings, refreshSettings, logActivity, memoryCount, refreshMemoryCount } =
    useEdith();
  const [provider, setProvider] = useState<AiProviderId>("openai");
  const [model, setModel] = useState(defaultModels.openai);
  const [apiKey, setApiKey] = useState("");
  const [showKey, setShowKey] = useState(false);
  const [status, setStatus] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [memoryKey, setMemoryKey] = useState("");
  const [memoryValue, setMemoryValue] = useState("");

  useEffect(() => {
    if (!settings) {
      return;
    }
    setProvider(settings.provider);
    setModel(settings.model || defaultModels[settings.provider]);
  }, [settings]);

  async function saveProvider(event: FormEvent) {
    event.preventDefault();
    setBusy(true);
    setError(null);
    try {
      await saveProviderSettings(provider, model.trim() || defaultModels[provider]);
      if (apiKey.trim()) {
        await saveApiKey(apiKey);
        setApiKey("");
      }
      await refreshSettings();
      logActivity("Settings opgeslagen");
      setStatus("Instellingen opgeslagen.");
    } catch (caught) {
      setError(toUserError(caught));
    } finally {
      setBusy(false);
    }
  }

  async function onClearKey() {
    setBusy(true);
    setError(null);
    try {
      await clearApiKey();
      setApiKey("");
      await refreshSettings();
      logActivity("API key verwijderd uit geheugen");
      setStatus("API key gewist uit runtime-geheugen.");
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
          API keys blijven in Rust-runtimegeheugen. Ze staan niet in de frontend-bundle.
        </p>
      </header>

      <form className="settings-form" onSubmit={saveProvider}>
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

        <div className="button-row">
          <button type="submit" disabled={busy}>
            Save
          </button>
          <button type="button" className="ghost" onClick={() => void onClearKey()} disabled={busy}>
            Clear
          </button>
        </div>
      </form>

      {status ? <p className="success-text">{status}</p> : null}
      {error ? (
        <p className="error-text" role="alert">
          {error}
        </p>
      ) : null}

      <article className="panel">
        <h3>Memory</h3>
        <p className="muted">
          Eenvoudige sessie-memory ({memoryCount} items). SQLite/embeddings volgen later.
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
      </article>

      <article className="panel">
        <h3>Voice</h3>
        <p className="muted">
          Spraak is in deze versie niet beschikbaar. Er is geen nep-microfoonstatus.
        </p>
      </article>

      <article className="panel">
        <h3>Security</h3>
        <ul className="plain-list">
          <li>Keys gaan via Tauri commands naar Rust-state, niet naar localStorage.</li>
          <li>De UI krijgt na opslaan alleen een gemaskeerde weergave.</li>
          <li>Desktop tools blijven uit tot er permissies en confirmatie zijn.</li>
        </ul>
      </article>
    </section>
  );
}
