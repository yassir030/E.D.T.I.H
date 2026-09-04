import type { ChatMessage } from "../types";
import { tauriAiClient, type AIProvider } from "./tauri";

export type { AIProvider };

export function getAiClient(): AIProvider {
  return tauriAiClient;
}

export async function sendAssistantMessage(
  messages: ChatMessage[],
): Promise<string> {
  return getAiClient().sendMessage(messages);
}
