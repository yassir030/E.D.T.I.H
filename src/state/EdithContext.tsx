import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type Dispatch,
  type ReactNode,
  type SetStateAction,
} from "react";
import { memoryService } from "../services/memory";
import { fetchPublicAiSettings, fetchSystemStatus } from "../services/tauri";
import type {
  ActivityItem,
  AppView,
  ChatMessage,
  CodingWorkspace,
  PublicAiSettings,
  SystemStatus,
  TaskItem,
} from "../types";
import { createId } from "../utils";

type EdithContextValue = {
  view: AppView;
  setView: (view: AppView) => void;
  messages: ChatMessage[];
  setMessages: Dispatch<SetStateAction<ChatMessage[]>>;
  tasks: TaskItem[];
  setTasks: Dispatch<SetStateAction<TaskItem[]>>;
  coding: CodingWorkspace;
  setCoding: Dispatch<SetStateAction<CodingWorkspace>>;
  activities: ActivityItem[];
  logActivity: (label: string) => void;
  settings: PublicAiSettings | null;
  systemStatus: SystemStatus | null;
  refreshSettings: () => Promise<void>;
  memoryCount: number;
  refreshMemoryCount: () => void;
};

const EdithContext = createContext<EdithContextValue | null>(null);

const defaultSettings: PublicAiSettings = {
  provider: "openai",
  model: "gpt-4o-mini",
  hasApiKey: false,
  maskedApiKey: null,
};

export function EdithProvider({ children }: { children: ReactNode }) {
  const [view, setViewState] = useState<AppView>("dashboard");
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [tasks, setTasks] = useState<TaskItem[]>([]);
  const [coding, setCoding] = useState<CodingWorkspace>({
    language: "typescript",
    code: "",
    output: "",
  });
  const [activities, setActivities] = useState<ActivityItem[]>([]);
  const [settings, setSettings] = useState<PublicAiSettings | null>(null);
  const [systemStatus, setSystemStatus] = useState<SystemStatus | null>(null);
  const [memoryCount, setMemoryCount] = useState(0);

  const logActivity = useCallback((label: string) => {
    setActivities((current) => [
      { id: createId(), label, timestamp: Date.now() },
      ...current,
    ].slice(0, 12));
  }, []);

  const setView = useCallback(
    (next: AppView) => {
      setViewState(next);
    },
    [],
  );

  const refreshMemoryCount = useCallback(() => {
    setMemoryCount(memoryService.getMemories().length);
  }, []);

  const refreshSettings = useCallback(async () => {
    try {
      const [nextSettings, nextStatus] = await Promise.all([
        fetchPublicAiSettings(),
        fetchSystemStatus(),
      ]);
      setSettings(nextSettings);
      setSystemStatus(nextStatus);
    } catch {
      setSettings((current) => current ?? defaultSettings);
    }
  }, []);

  useEffect(() => {
    void refreshSettings();
    refreshMemoryCount();
  }, [refreshMemoryCount, refreshSettings]);

  const value = useMemo(
    () => ({
      view,
      setView,
      messages,
      setMessages,
      tasks,
      setTasks,
      coding,
      setCoding,
      activities,
      logActivity,
      settings,
      systemStatus,
      refreshSettings,
      memoryCount,
      refreshMemoryCount,
    }),
    [
      activities,
      coding,
      logActivity,
      memoryCount,
      messages,
      refreshMemoryCount,
      refreshSettings,
      setView,
      settings,
      systemStatus,
      tasks,
      view,
    ],
  );

  return <EdithContext.Provider value={value}>{children}</EdithContext.Provider>;
}

export function useEdith(): EdithContextValue {
  const value = useContext(EdithContext);
  if (!value) {
    throw new Error("useEdith must be used within EdithProvider");
  }
  return value;
}
