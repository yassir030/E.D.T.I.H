import type { AppView } from "../types";

const items: { view: AppView; label: string }[] = [
  { view: "dashboard", label: "Dashboard" },
  { view: "assistant", label: "Assistant" },
  { view: "tasks", label: "Tasks" },
  { view: "coding", label: "Coding" },
  { view: "files", label: "Files" },
  { view: "settings", label: "Settings" },
];

type SidebarProps = {
  activeView: AppView;
  onNavigate: (view: AppView) => void;
  aiReady: boolean;
};

export function Sidebar({ activeView, onNavigate, aiReady }: SidebarProps) {
  return (
    <aside className="sidebar">
      <h1 className="logo">E.D.I.T.H.</h1>

      <nav className="sidebar-nav" aria-label="Hoofdnavigatie">
        {items.map((item) => {
          const selected = item.view === activeView;
          return (
            <button
              key={item.view}
              type="button"
              className={selected ? "nav-btn active" : "nav-btn"}
              aria-current={selected ? "page" : undefined}
              onClick={() => onNavigate(item.view)}
            >
              {item.label}
            </button>
          );
        })}
      </nav>

      <div
        className={aiReady ? "online" : "online offline"}
        role="status"
        aria-live="polite"
      >
        {aiReady ? "● AI Core Online" : "● AI Core Offline"}
      </div>
    </aside>
  );
}
