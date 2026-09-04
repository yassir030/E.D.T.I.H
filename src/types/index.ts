export type AppView =
  | "dashboard"
  | "assistant"
  | "tasks"
  | "coding"
  | "files"
  | "settings";

export type AiProviderId = "openai" | "gemini" | "claude";

export type ChatRole = "user" | "assistant" | "system";

export type ChatStatus = "idle" | "sending" | "success" | "error";

export type ChatMessage = {
  id: string;
  role: ChatRole;
  content: string;
  timestamp: number;
};

export type TaskItem = {
  id: string;
  title: string;
  completed: boolean;
  createdAt: number;
  updatedAt: number;
};

export type MemoryRecord = {
  id: string;
  key: string;
  value: string;
  createdAt: number;
};

export type PublicAiSettings = {
  provider: AiProviderId;
  model: string;
  hasApiKey: boolean;
  maskedApiKey: string | null;
};

export type SystemStatus = {
  appName: string;
  version: string;
  platform: string;
  filesystemReady: boolean;
  memoryBackend: string;
  voiceReady: boolean;
};

export type DesktopTool = {
  id: string;
  name: string;
  description: string;
  input: string;
  output: string;
  requiresConfirmation: boolean;
  enabled: boolean;
};

export type FilesystemStatus = {
  available: boolean;
  message: string;
};

export type ActivityItem = {
  id: string;
  label: string;
  timestamp: number;
};

export type CodingLanguage = "javascript" | "typescript" | "python" | "rust" | "other";

export type CodingWorkspace = {
  language: CodingLanguage;
  code: string;
  output: string;
};
