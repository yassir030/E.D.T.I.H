import {
  useEffect,
  useRef,
  useState,
  type FormEvent,
  type KeyboardEvent,
} from "react";
import { sendAssistantMessage } from "../services/ai";
import { useEdith } from "../state/EdithContext";
import type { ChatMessage, ChatStatus } from "../types";
import { createId, toUserError } from "../utils";

export function AssistantView() {
  const {
    messages,
    setMessages,
    settings,
    setView,
    logActivity,
  } = useEdith();
  const [draft, setDraft] = useState("");
  const [status, setStatus] = useState<ChatStatus>("idle");
  const [error, setError] = useState<string | null>(null);
  const listRef = useRef<HTMLDivElement | null>(null);
  const sendingRef = useRef(false);

  const providerReady = Boolean(settings?.hasApiKey);
  const busy = status === "sending";

  useEffect(() => {
    const node = listRef.current;
    if (!node) {
      return;
    }
    node.scrollTop = node.scrollHeight;
  }, [messages, status]);

  async function submit() {
    const content = draft.trim();
    if (!content || sendingRef.current) {
      return;
    }
    if (!providerReady) {
      setError("Configureer eerst een AI provider in Settings.");
      setStatus("error");
      return;
    }

    sendingRef.current = true;
    setError(null);
    setStatus("sending");

    const userMessage: ChatMessage = {
      id: createId(),
      role: "user",
      content,
      timestamp: Date.now(),
    };
    const nextMessages = [...messages, userMessage];
    setMessages(nextMessages);
    setDraft("");
    logActivity("Bericht verzonden naar Assistant");

    try {
      const reply = await sendAssistantMessage(nextMessages);
      const assistantMessage: ChatMessage = {
        id: createId(),
        role: "assistant",
        content: reply,
        timestamp: Date.now(),
      };
      setMessages((current) => [...current, assistantMessage]);
      setStatus("success");
    } catch (caught) {
      setStatus("error");
      setError(toUserError(caught));
    } finally {
      sendingRef.current = false;
    }
  }

  function onSubmit(event: FormEvent) {
    event.preventDefault();
    void submit();
  }

  function onKeyDown(event: KeyboardEvent<HTMLTextAreaElement>) {
    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      void submit();
    }
  }

  function clearConversation() {
    setMessages([]);
    setError(null);
    setStatus("idle");
    setDraft("");
    logActivity("Assistant-gesprek gewist");
  }

  return (
    <section className="view chat-view">
      <header className="page-header">
        <h2>Assistant</h2>
        <p className="subtitle">Praat met E.D.I.T.H. via je geconfigureerde provider.</p>
        <div className="button-row">
          <button
            type="button"
            className="ghost"
            onClick={clearConversation}
            disabled={busy || (messages.length === 0 && !draft && !error)}
          >
            Clear conversation
          </button>
        </div>
      </header>

      {!providerReady ? (
        <div className="banner" role="status">
          <p>Configureer eerst een AI provider in Settings.</p>
          <button type="button" onClick={() => setView("settings")}>
            Open Settings
          </button>
        </div>
      ) : null}

      <div className="chat-layout">
        <div
          className="messages"
          ref={listRef}
          role="log"
          aria-live="polite"
          aria-relevant="additions"
        >
          {messages.length === 0 ? (
            <div className="empty-state">
              <p>Nog geen berichten. Typ hieronder om te beginnen.</p>
            </div>
          ) : (
            messages
              .filter((message) => message.role !== "system")
              .map((message) => (
                <article
                  key={message.id}
                  className={`bubble ${message.role}`}
                >
                  <p>{message.content}</p>
                </article>
              ))
          )}
          {busy ? (
            <article className="bubble assistant pending">
              <p>E.D.I.T.H. denkt na…</p>
            </article>
          ) : null}
        </div>

        {error ? (
          <p className="error-text" role="alert">
            {error}
          </p>
        ) : null}

        <form className="composer" onSubmit={onSubmit}>
          <label htmlFor="assistant-input" className="sr-only">
            Bericht aan E.D.I.T.H.
          </label>
          <textarea
            id="assistant-input"
            value={draft}
            onChange={(event) => setDraft(event.target.value)}
            onKeyDown={onKeyDown}
            placeholder="Talk to E.D.I.T.H..."
            rows={3}
            disabled={busy}
          />
          <button type="submit" disabled={busy || !draft.trim()}>
            {busy ? "Sending…" : "Send"}
          </button>
        </form>
      </div>
    </section>
  );
}
