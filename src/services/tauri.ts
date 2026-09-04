import { invoke } from "@tauri-apps/api/core";
import type {
  AiProviderId,
  ChatMessage,
  DesktopTool,
  FilesystemStatus,
  PublicAiSettings,
  SystemStatus,
} from "../types";

type RustPublicSettings = {
  provider: string;
  model: string;
  has_api_key: boolean;
  masked_api_key: string | null;
};

type RustSystemStatus = {
  app_name: string;
  version: string;
  platform: string;
  filesystem_ready: boolean;
  memory_backend: string;
  voice_ready: boolean;
};

type RustDesktopTool = {
  id: string;
  name: string;
  description: string;
  input: string;
  output: string;
  requires_confirmation: boolean;
  enabled: boolean;
};

type RustFilesystemStatus = {
  available: boolean;
  message: string;
};

function mapProvider(value: string): AiProviderId {
  if (value === "gemini" || value === "claude" || value === "openai") {
    return value;
  }
  return "openai";
}

function mapSettings(raw: RustPublicSettings): PublicAiSettings {
  return {
    provider: mapProvider(raw.provider),
    model: raw.model,
    hasApiKey: raw.has_api_key,
    maskedApiKey: raw.masked_api_key,
  };
}

export async function fetchPublicAiSettings(): Promise<PublicAiSettings> {
  const raw = await invoke<RustPublicSettings>("get_public_ai_settings");
  return mapSettings(raw);
}

export async function saveProviderSettings(
  provider: AiProviderId,
  model: string,
): Promise<PublicAiSettings> {
  const raw = await invoke<RustPublicSettings>("save_provider_settings", {
    payload: { provider, model },
  });
  return mapSettings(raw);
}

export async function saveApiKey(apiKey: string): Promise<PublicAiSettings> {
  const raw = await invoke<RustPublicSettings>("save_api_key", {
    payload: { api_key: apiKey },
  });
  return mapSettings(raw);
}

export async function clearApiKey(): Promise<PublicAiSettings> {
  const raw = await invoke<RustPublicSettings>("clear_api_key");
  return mapSettings(raw);
}

export async function fetchSystemStatus(): Promise<SystemStatus> {
  const raw = await invoke<RustSystemStatus>("get_system_status");
  return {
    appName: raw.app_name,
    version: raw.version,
    platform: raw.platform,
    filesystemReady: raw.filesystem_ready,
    memoryBackend: raw.memory_backend,
    voiceReady: raw.voice_ready,
  };
}

export async function fetchDesktopTools(): Promise<DesktopTool[]> {
  const raw = await invoke<RustDesktopTool[]>("list_desktop_tools");
  return raw.map((tool) => ({
    id: tool.id,
    name: tool.name,
    description: tool.description,
    input: tool.input,
    output: tool.output,
    requiresConfirmation: tool.requires_confirmation,
    enabled: tool.enabled,
  }));
}

export async function fetchFilesystemStatus(): Promise<FilesystemStatus> {
  return invoke<RustFilesystemStatus>("get_filesystem_status");
}

export async function invokeDesktopTool(toolId: string): Promise<string> {
  return invoke<string>("invoke_desktop_tool", { toolId });
}

export type AIProvider = {
  sendMessage: (messages: ChatMessage[]) => Promise<string>;
  streamMessage: (
    messages: ChatMessage[],
    onChunk: (chunk: string) => void,
  ) => Promise<string>;
};

async function sendAiMessage(messages: ChatMessage[]): Promise<string> {
  const result = await invoke<{ content: string }>("send_ai_message", {
    payload: {
      messages: messages.map((message) => ({
        role: message.role,
        content: message.content,
      })),
    },
  });
  return result.content;
}

export const tauriAiClient: AIProvider = {
  async sendMessage(messages) {
    return sendAiMessage(messages);
  },
  async streamMessage(messages, onChunk) {
    const content = await sendAiMessage(messages);
    onChunk(content);
    return content;
  },
};
