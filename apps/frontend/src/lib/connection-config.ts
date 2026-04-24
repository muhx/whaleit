const STORAGE_KEY = "whaleit-api-connection";

export interface ConnectionConfig {
  apiHost: string;
  apiKey: string;
}

export function getConnectionConfig(): ConnectionConfig | null {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return null;
    return JSON.parse(raw) as ConnectionConfig;
  } catch {
    return null;
  }
}

export function setConnectionConfig(config: ConnectionConfig): void {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(config));
}

export function clearConnectionConfig(): void {
  localStorage.removeItem(STORAGE_KEY);
}

export async function testConnection(
  config: ConnectionConfig,
): Promise<{ success: boolean; error?: string }> {
  try {
    const url = config.apiHost.replace(/\/+$/, "");
    const res = await fetch(`${url}/api/v1/auth/me`, {
      headers: { Authorization: `Bearer ${config.apiKey}` },
      signal: AbortSignal.timeout(10_000),
    });
    if (res.ok) {
      return { success: true };
    }
    if (res.status === 401 || res.status === 403) {
      return { success: false, error: "Invalid API key" };
    }
    return { success: false, error: `Server returned ${res.status}` };
  } catch (e) {
    return { success: false, error: e instanceof Error ? e.message : "Connection failed" };
  }
}
