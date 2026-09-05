import { useEffect, useState } from "react";
import {
  fetchFilesystemStatus,
  invokeDesktopTool,
} from "../services/tauri";
import type { FilesystemStatus } from "../types";
import { toUserError } from "../utils";

// Safe current directory for browser context
const CURRENT_DIR = ".";

type FileItem = {
  name: string;
  path: string;
  isFile: boolean;
  isDir: boolean;
  size: number;
  modified?: number;
};

export function FilesView() {
  const [fsStatus, setFsStatus] = useState<FilesystemStatus | null>(null);
  const [currentPath, setCurrentPath] = useState<string>("");
  const [files, setFiles] = useState<FileItem[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    loadFilesystemStatus();
    // Start with a safe default directory
    setCurrentPath(CURRENT_DIR);
  }, []);

  async function loadFilesystemStatus() {
    try {
      const status = await fetchFilesystemStatus();
      setFsStatus(status);
    } catch (error) {
      setFsStatus({
        available: false,
        message: toUserError(error),
      });
    }
  }

  async function loadDirectory(path: string) {
    setLoading(true);
    setError(null);
    try {
      const result = await invokeDesktopTool("list_directory", { path });
      if (result.success && result.data) {
        const items = (result.data as { items: FileItem[] }).items;
        setFiles(items);
        setCurrentPath(path);
      } else {
        setError(result.message);
      }
    } catch (error) {
      setError(toUserError(error));
    } finally {
      setLoading(false);
    }
  }

  function goUp() {
    const parentPath = currentPath.split(/[/\\]/).slice(0, -1).join("/");
    if (parentPath) {
      loadDirectory(parentPath);
    }
  }

  function goHome() {
    // For now, just go to current directory
    loadDirectory(".");
  }

  function formatSize(bytes: number): string {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return Math.round(bytes / Math.pow(k, i) * 100) / 100 + " " + sizes[i];
  }

  function formatDate(timestamp?: number): string {
    if (!timestamp) return "-";
    return new Date(timestamp * 1000).toLocaleDateString();
  }

  return (
    <section className="view">
      <header className="page-header">
        <h2>Files</h2>
        <p className="subtitle">Real filesystem access with permission controls.</p>
        <div className="button-row">
          <button type="button" className="ghost" onClick={goHome} disabled={loading}>
            Home
          </button>
          <button type="button" className="ghost" onClick={goUp} disabled={loading || currentPath === "."}>
            Up
          </button>
        </div>
      </header>

      <article className="panel">
        <h3>Filesystem Status</h3>
        <p className="muted">
          {fsStatus?.message ?? "Loading status..."}
        </p>
      </article>

      <div className="file-browser">
        <div className="file-controls">
          <input
            type="text"
            value={currentPath}
            onChange={(e) => setCurrentPath(e.target.value)}
            placeholder="Enter path..."
            className="path-input"
          />
          <button onClick={() => loadDirectory(currentPath)} disabled={loading}>
            Browse
          </button>
        </div>

        {error && <p className="error-text">{error}</p>}

        {loading ? (
          <p className="muted">Loading directory...</p>
        ) : (
          <div className="file-list">
            {files.length === 0 ? (
              <p className="muted">No files in this directory</p>
            ) : (
              <table className="file-table">
                <thead>
                  <tr>
                    <th>Name</th>
                    <th>Type</th>
                    <th>Size</th>
                    <th>Modified</th>
                  </tr>
                </thead>
                <tbody>
                  {files.map((file) => (
                    <tr
                      key={file.path}
                      onClick={() => file.isDir && loadDirectory(file.path)}
                      className={file.isDir ? "directory-row" : "file-row"}
                    >
                      <td>{file.name}</td>
                      <td>{file.isDir ? "Directory" : "File"}</td>
                      <td>{file.isFile ? formatSize(file.size) : "-"}</td>
                      <td>{formatDate(file.modified)}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            )}
          </div>
        )}
      </div>
    </section>
  );
}
