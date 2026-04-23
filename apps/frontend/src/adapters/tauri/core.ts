import { debug, error, info, trace, warn } from "@tauri-apps/plugin-log";

import { getConnectionConfig, clearConnectionConfig } from "@/lib/connection-config";
import type { Logger } from "../types";
import {
  COMMANDS,
  API_PREFIX as WEB_API_PREFIX,
  EVENTS_ENDPOINT as _EVENTS_ENDPOINT,
  AI_CHAT_STREAM_ENDPOINT as _AI_CHAT_STREAM_ENDPOINT,
  fromBase64,
  handleCommand,
} from "../web/core";

export const logger: Logger = {
  error: (...args: unknown[]) => {
    error(args.map(String).join(" "));
  },
  warn: (...args: unknown[]) => {
    warn(args.map(String).join(" "));
  },
  info: (...args: unknown[]) => {
    info(args.map(String).join(" "));
  },
  debug: (...args: unknown[]) => {
    debug(args.map(String).join(" "));
  },
  trace: (...args: unknown[]) => {
    trace(args.map(String).join(" "));
  },
};

export const isDesktop = true;
export const isWeb = false;

export const API_PREFIX = WEB_API_PREFIX;
export const EVENTS_ENDPOINT = _EVENTS_ENDPOINT;
export const AI_CHAT_STREAM_ENDPOINT = _AI_CHAT_STREAM_ENDPOINT;

export const invoke = async <T>(
  command: string,
  payload?: Record<string, unknown>,
): Promise<T> => {
  const config = COMMANDS[command];
  if (!config) throw new Error(`Unsupported command ${command}`);

  const connection = getConnectionConfig();
  if (!connection) {
    throw new Error("No API connection configured");
  }

  const baseUrl = connection.apiHost.replace(/\/+$/, "");
  let url = `${baseUrl}${API_PREFIX}${config.path}`;

  const result = handleCommand(command, url, payload);
  url = result.url;
  const body = result.body;

  const headers: HeadersInit = {
    Authorization: `Bearer ${connection.apiKey}`,
  };
  if (body !== undefined) {
    headers["Content-Type"] = "application/json";
  }

  const res = await fetch(url, {
    method: config.method,
    headers,
    body,
    signal: AbortSignal.timeout(300_000),
  });

  if (res.status === 401) {
    clearConnectionConfig();
    throw new Error("API key is invalid. Please reconnect.");
  }
  if (!res.ok) {
    let msg = res.statusText;
    try {
      const err = await res.json();
      msg = (err?.message ?? msg) as string;
    } catch {
      void 0;
    }
    console.error(`[Invoke] Command "${command}" failed: ${msg}`);
    throw new Error(msg);
  }
  if (command === "backup_database") {
    const parsed = (await res.json()) as { filename: string; dataB64: string };
    return {
      filename: parsed.filename,
      data: fromBase64(parsed.dataB64),
    } as T;
  }
  if (command === "backup_database_to_path") {
    const parsed = (await res.json()) as { path: string };
    return parsed.path as T;
  }
  if (res.status === 204 || res.status === 202) {
    return undefined as T;
  }
  const text = await res.text();
  if (!text) {
    return undefined as T;
  }
  return JSON.parse(text) as T;
};
