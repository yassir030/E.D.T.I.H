export function createId(): string {
  if (typeof crypto !== "undefined" && "randomUUID" in crypto) {
    return crypto.randomUUID();
  }
  return `id_${Date.now()}_${Math.random().toString(16).slice(2)}`;
}

export function formatTime(timestamp: number): string {
  return new Date(timestamp).toLocaleTimeString([], {
    hour: "2-digit",
    minute: "2-digit",
  });
}

export function toUserError(error: unknown): string {
  let message = "Er ging iets mis. Probeer het opnieuw.";
  if (typeof error === "string" && error.trim()) {
    message = error;
  } else if (error instanceof Error && error.message.trim()) {
    message = error.message;
  }
  return redactSecrets(message);
}

function redactSecrets(message: string): string {
  return message
    .replace(/sk-[A-Za-z0-9_-]+/g, "[redacted]")
    .replace(/AIza[A-Za-z0-9_-]+/g, "[redacted]")
    .replace(/sk-ant-[A-Za-z0-9_-]+/g, "[redacted]");
}
