import "./App.css";
import { Sidebar } from "./components/Sidebar";
import { EdithProvider, useEdith } from "./state/EdithContext";
import { AssistantView } from "./views/AssistantView";
import { CodingView } from "./views/CodingView";
import { DashboardView } from "./views/DashboardView";
import { FilesView } from "./views/FilesView";
import { SettingsView } from "./views/SettingsView";
import { TasksView } from "./views/TasksView";

function AppShell() {
  const { view, setView, settings } = useEdith();

  return (
    <div className="edith">
      <Sidebar
        activeView={view}
        onNavigate={setView}
        aiReady={Boolean(settings?.hasApiKey)}
      />
      <main className="main">
        {view === "dashboard" ? <DashboardView /> : null}
        {view === "assistant" ? <AssistantView /> : null}
        {view === "tasks" ? <TasksView /> : null}
        {view === "coding" ? <CodingView /> : null}
        {view === "files" ? <FilesView /> : null}
        {view === "settings" ? <SettingsView /> : null}
      </main>
    </div>
  );
}

function App() {
  return (
    <EdithProvider>
      <AppShell />
    </EdithProvider>
  );
}

export default App;
