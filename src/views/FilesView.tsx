import { useEffect, useState } from "react";
import {
  fetchDesktopTools,
  fetchFilesystemStatus,
  invokeDesktopTool,
} from "../services/tauri";
import type { DesktopTool, FilesystemStatus } from "../types";
import { toUserError } from "../utils";

export function FilesView() {
  const [fsStatus, setFsStatus] = useState<FilesystemStatus | null>(null);
  const [tools, setTools] = useState<DesktopTool[]>([]);
  const [toolMessage, setToolMessage] = useState<string | null>(null);

  useEffect(() => {
    let active = true;
    void (async () => {
      try {
        const [status, desktopTools] = await Promise.all([
          fetchFilesystemStatus(),
          fetchDesktopTools(),
        ]);
        if (!active) {
          return;
        }
        setFsStatus(status);
        setTools(desktopTools);
      } catch (error) {
        if (active) {
          setFsStatus({
            available: false,
            message: toUserError(error),
          });
        }
      }
    })();
    return () => {
      active = false;
    };
  }, []);

  async function tryTool(toolId: string) {
    setToolMessage(null);
    try {
      const result = await invokeDesktopTool(toolId);
      setToolMessage(result);
    } catch (error) {
      setToolMessage(toUserError(error));
    }
  }

  return (
    <section className="view">
      <header className="page-header">
        <h2>Files</h2>
        <p className="subtitle">Voorbereide filesystem- en tool-laag. Geen nepbestanden.</p>
      </header>

      <article className="panel">
        <h3>Filesystem</h3>
        <p className="muted">
          {fsStatus?.message ?? "Status wordt opgevraagd bij de Tauri-backend…"}
        </p>
      </article>

      <article className="panel">
        <h3>Desktop tools (permissions)</h3>
        <p className="muted">
          Elke tool heeft een naam, input en output. Destructieve acties vereisen later
          bevestiging. Niets wordt nu uitgevoerd.
        </p>
        <ul className="tool-list">
          {tools.map((tool) => (
            <li key={tool.id}>
              <div>
                <strong>{tool.name}</strong>
                <span>
                  input: {tool.input} · output: {tool.output}
                  {tool.requiresConfirmation ? " · confirmation required" : ""}
                </span>
              </div>
              <button type="button" className="ghost" onClick={() => void tryTool(tool.id)}>
                Probeer
              </button>
            </li>
          ))}
        </ul>
        {toolMessage ? <p className="error-text">{toolMessage}</p> : null}
      </article>
    </section>
  );
}
