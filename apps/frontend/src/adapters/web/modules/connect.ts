// Connect (broker sync) domain handlers

export function handleStoreSyncSession(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { refreshToken } = payload as { refreshToken: string };
  return { url, body: JSON.stringify({ refreshToken }) };
}

export function handleGetImportRuns(
  url: string,
  payload?: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { runType, limit, offset } = (payload ?? {}) as {
    runType?: string;
    limit?: number;
    offset?: number;
  };
  const params = new URLSearchParams();
  if (runType) params.set("runType", runType);
  if (limit !== undefined) params.set("limit", String(limit));
  if (offset !== undefined) params.set("offset", String(offset));
  const qs = params.toString();
  return { url: qs ? `${url}?${qs}` : url, body: undefined };
}

export function handleGetBrokerSyncProfile(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { accountId, sourceSystem } = payload as { accountId: string; sourceSystem: string };
  const params = new URLSearchParams();
  params.set("accountId", accountId);
  params.set("sourceSystem", sourceSystem);
  return { url: `${url}?${params.toString()}`, body: undefined };
}

export function handleSaveBrokerSyncProfileRules(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { request } = payload as { request: Record<string, unknown> };
  return { url, body: JSON.stringify(request) };
}

export function handleDeviceSyncReconcileReadyState(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  return { url, body: JSON.stringify(payload ?? {}) };
}
