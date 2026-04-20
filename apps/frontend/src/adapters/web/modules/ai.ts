// AI providers, threads, messages, models, tags, chat domain handlers

export function handleUpdateAiProviderSettings(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { request } = payload as { request: Record<string, unknown> };
  return { url, body: JSON.stringify(request) };
}

export function handleSetDefaultAiProvider(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { request } = payload as { request: Record<string, unknown> };
  return { url, body: JSON.stringify(request) };
}

export function handleListAiModels(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { providerId } = payload as { providerId: string };
  return { url: `${url}/${encodeURIComponent(providerId)}/models`, body: undefined };
}

export function handleListAiThreads(
  url: string,
  payload?: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { cursor, limit, search } = (payload ?? {}) as {
    cursor?: string;
    limit?: number;
    search?: string;
  };
  const params = new URLSearchParams();
  if (cursor) params.set("cursor", cursor);
  if (limit !== undefined) params.set("limit", String(limit));
  if (search) params.set("search", search);
  const qs = params.toString();
  return { url: qs ? `${url}?${qs}` : url, body: undefined };
}

export function handleGetAiThread(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { threadId } = payload as { threadId: string };
  return { url: `${url}/${encodeURIComponent(threadId)}`, body: undefined };
}

export function handleGetAiThreadMessages(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { threadId } = payload as { threadId: string };
  return { url: `${url}/${encodeURIComponent(threadId)}/messages`, body: undefined };
}

export function handleUpdateToolResult(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { request } = payload as {
    request: { threadId: string; toolCallId: string; resultPatch: unknown };
  };
  return {
    url,
    body: JSON.stringify({
      threadId: request.threadId,
      toolCallId: request.toolCallId,
      resultPatch: request.resultPatch,
    }),
  };
}

export function handleUpdateAiThread(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { request } = payload as { request: { id: string; title?: string; isPinned?: boolean } };
  return {
    url: `${url}/${encodeURIComponent(request.id)}`,
    body: JSON.stringify({ title: request.title, isPinned: request.isPinned }),
  };
}

export function handleDeleteAiThread(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { threadId } = payload as { threadId: string };
  return { url: `${url}/${encodeURIComponent(threadId)}`, body: undefined };
}

export function handleAddAiThreadTag(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { threadId, tag } = payload as { threadId: string; tag: string };
  return { url: `${url}/${encodeURIComponent(threadId)}/tags`, body: JSON.stringify({ tag }) };
}

export function handleRemoveAiThreadTag(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { threadId, tag } = payload as { threadId: string; tag: string };
  return { url: `${url}/${encodeURIComponent(threadId)}/tags/${encodeURIComponent(tag)}`, body: undefined };
}

export function handleGetAiThreadTags(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { threadId } = payload as { threadId: string };
  return { url: `${url}/${encodeURIComponent(threadId)}/tags`, body: undefined };
}
