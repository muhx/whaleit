// Web adapter - AI Chat Streaming (platform-specific HTTP implementation)

import { logger, API_PREFIX } from "./core";
import { getConnectionConfig } from "@/lib/connection-config";
import type { AiSendMessageRequest, AiStreamEvent } from "@/features/ai-assistant/types";

function resolveEndpoint(path: string): string {
  const connection = getConnectionConfig();
  if (connection) {
    return `${connection.apiHost.replace(/\/+$/, "")}${path}`;
  }
  return path;
}

function buildAuthHeaders(): Record<string, string> {
  const connection = getConnectionConfig();
  if (connection) {
    return { Authorization: `Bearer ${connection.apiKey}` };
  }
  return {};
}

/**
 * Stream AI chat responses via HTTP fetch.
 *
 * Uses NDJSON streaming for efficient event delivery.
 *
 * @param request - The chat message request
 * @param signal - Optional AbortSignal for cancellation
 * @yields AiStreamEvent objects from the stream
 */
export async function* streamAiChat(
  request: AiSendMessageRequest,
  signal?: AbortSignal,
): AsyncGenerator<AiStreamEvent, void, undefined> {
  const response = await fetch(resolveEndpoint(`${API_PREFIX}/ai/chat/stream`), {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      ...buildAuthHeaders(),
    },
    body: JSON.stringify(request),
    signal,
    ...(getConnectionConfig() ? {} : { credentials: "same-origin" as RequestCredentials }),
  });

  if (!response.ok) {
    let errorMessage = response.statusText;
    let errorCode = "network";

    try {
      const errorBody = (await response.json()) as { code?: string; error?: string };
      errorCode = errorBody.code ?? "network";
      errorMessage = errorBody.error ?? errorMessage;
    } catch {
      // Ignore JSON parse error
    }

    yield {
      type: "error",
      threadId: "",
      runId: "",
      messageId: undefined,
      code: errorCode,
      message: errorMessage,
    } as AiStreamEvent;
    return;
  }

  if (!response.body) {
    yield {
      type: "error",
      threadId: "",
      runId: "",
      messageId: undefined,
      code: "network",
      message: "Response body is null",
    } as AiStreamEvent;
    return;
  }

  // Parse NDJSON stream
  const reader = response.body.getReader();
  const decoder = new TextDecoder();
  let buffer = "";

  try {
    while (true) {
      const { done, value } = await reader.read();

      if (done) {
        // Process any remaining buffer content
        if (buffer.trim()) {
          try {
            const event = JSON.parse(buffer.trim()) as AiStreamEvent;
            yield event;
          } catch (parseError) {
            logger.error("Failed to parse final buffer:", parseError);
          }
        }
        break;
      }

      // Decode chunk and add to buffer
      buffer += decoder.decode(value, { stream: true });

      // Split by newlines and process complete lines
      const lines = buffer.split("\n");

      // Keep the last incomplete line in the buffer
      buffer = lines.pop() ?? "";

      for (const line of lines) {
        const trimmed = line.trim();
        if (!trimmed) continue;

        try {
          const event = JSON.parse(trimmed) as AiStreamEvent;
          yield event;

          // Stop on terminal events
          if (event.type === "done" || event.type === "error") {
            return;
          }
        } catch (parseError) {
          logger.error("Failed to parse NDJSON line:", trimmed, parseError);
        }
      }
    }
  } finally {
    reader.releaseLock();
  }
}
