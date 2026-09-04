import { sendAssistantMessage } from "../services/ai";
import { useEdith } from "../state/EdithContext";
import type { ChatMessage, CodingLanguage } from "../types";
import { createId, toUserError } from "../utils";

const languages: { id: CodingLanguage; label: string }[] = [
  { id: "typescript", label: "TypeScript" },
  { id: "javascript", label: "JavaScript" },
  { id: "python", label: "Python" },
  { id: "rust", label: "Rust" },
  { id: "other", label: "Other" },
];

export function CodingView() {
  const { coding, setCoding, settings, setView, logActivity } = useEdith();
  const providerReady = Boolean(settings?.hasApiKey);

  function clearWorkspace() {
    setCoding((current) => ({ ...current, code: "", output: "" }));
  }

  async function runAi(action: "analyze" | "explain" | "fix" | "improve" | "generate") {
    if (!providerReady) {
      setCoding((current) => ({
        ...current,
        output:
          "Configureer eerst een AI provider in Settings. Lokale code wordt niet op het systeem uitgevoerd.",
      }));
      return;
    }
    if (!coding.code.trim() && action !== "generate") {
      setCoding((current) => ({
        ...current,
        output: "Plak of typ eerst code in de editor.",
      }));
      return;
    }

    const prompt = buildPrompt(action, coding.language, coding.code);
    const messages: ChatMessage[] = [
      {
        id: createId(),
        role: "user",
        content: prompt,
        timestamp: Date.now(),
      },
    ];

    setCoding((current) => ({
      ...current,
      output: "AI-analyse bezig… Code wordt niet lokaal uitgevoerd.",
    }));
    logActivity(`Coding: ${action}`);

    try {
      const reply = await sendAssistantMessage(messages);
      setCoding((current) => ({ ...current, output: reply }));
    } catch (error) {
      setCoding((current) => ({ ...current, output: toUserError(error) }));
    }
  }

  return (
    <section className="view">
      <header className="page-header">
        <h2>Coding</h2>
        <p className="subtitle">
          Workspace voor code. Er worden geen systeemcommando’s uitgevoerd.
        </p>
      </header>

      {!providerReady ? (
        <div className="banner">
          <p>AI coding-assistentie vereist een provider.</p>
          <button type="button" onClick={() => setView("settings")}>
            Open Settings
          </button>
        </div>
      ) : null}

      <div className="coding-toolbar">
        <label htmlFor="language">Taal</label>
        <select
          id="language"
          value={coding.language}
          onChange={(event) => {
            const next = event.target.value;
            const match = languages.find((language) => language.id === next);
            if (!match) {
              return;
            }
            setCoding((current) => ({
              ...current,
              language: match.id,
            }));
          }}
        >
          {languages.map((language) => (
            <option key={language.id} value={language.id}>
              {language.label}
            </option>
          ))}
        </select>
        <button type="button" onClick={() => void runAi("analyze")}>
          Run
        </button>
        <button type="button" className="ghost" onClick={clearWorkspace}>
          Clear
        </button>
      </div>

      <div className="coding-actions">
        <button type="button" onClick={() => void runAi("explain")}>
          Explain code
        </button>
        <button type="button" onClick={() => void runAi("fix")}>
          Fix code
        </button>
        <button type="button" onClick={() => void runAi("improve")}>
          Improve code
        </button>
        <button type="button" onClick={() => void runAi("generate")}>
          Generate code
        </button>
      </div>

      <label htmlFor="code-editor" className="sr-only">
        Code editor
      </label>
      <textarea
        id="code-editor"
        className="code-editor"
        value={coding.code}
        onChange={(event) =>
          setCoding((current) => ({ ...current, code: event.target.value }))
        }
        placeholder="Schrijf of plak hier code…"
        spellCheck={false}
      />

      <section className="output-panel">
        <h3>Output / errors</h3>
        <pre>{coding.output || "Nog geen output."}</pre>
      </section>
    </section>
  );
}

function buildPrompt(
  action: "analyze" | "explain" | "fix" | "improve" | "generate",
  language: CodingLanguage,
  code: string,
): string {
  if (action === "generate") {
    return `Generate ${language} code for the following request. Return only the code and a short explanation.\n\n${code || "Create a small, useful example."}`;
  }

  const instruction =
    action === "analyze"
      ? "Do not execute anything. Explain the expected result and possible errors."
      : action === "explain"
        ? "Explain this code clearly."
        : action === "fix"
          ? "Fix bugs in this code and explain the changes."
          : "Improve this code and explain why.";

  return `${instruction}\nLanguage: ${language}\n\n${code}`;
}
