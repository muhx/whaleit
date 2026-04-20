// Activities domain handlers

export function handleSearchActivities(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  return { url, body: JSON.stringify(payload) };
}

export function handleCreateActivity(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { activity } = payload as { activity: Record<string, unknown> };
  return { url, body: JSON.stringify(activity) };
}

export function handleUpdateActivity(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { activity } = payload as { activity: Record<string, unknown> };
  return { url, body: JSON.stringify(activity) };
}

export function handleSaveActivities(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { request } = payload as { request: Record<string, unknown> };
  return { url, body: JSON.stringify(request) };
}

export function handleDeleteActivity(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { activityId } = payload as { activityId: string };
  return { url: `${url}/${encodeURIComponent(activityId)}`, body: undefined };
}

export function handleCheckActivitiesImport(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  return { url, body: JSON.stringify(payload) };
}

export function handlePreviewImportAssets(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  return { url, body: JSON.stringify(payload) };
}

export function handleImportActivities(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  return { url, body: JSON.stringify(payload) };
}

export function handleGetAccountImportMapping(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { accountId, contextKind } = payload as { accountId: string; contextKind?: string };
  const params = new URLSearchParams();
  params.set("accountId", accountId);
  if (contextKind) params.set("contextKind", contextKind);
  return { url: `${url}?${params.toString()}`, body: undefined };
}

export function handleSaveAccountImportMapping(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { mapping } = payload as { mapping: Record<string, unknown> };
  return { url, body: JSON.stringify({ mapping }) };
}

export function handleGetImportTemplate(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { id } = payload as { id: string };
  const params = new URLSearchParams();
  params.set("id", id);
  return { url: `${url}?${params.toString()}`, body: undefined };
}

export function handleDeleteImportTemplate(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { id } = payload as { id: string };
  const params = new URLSearchParams();
  params.set("id", id);
  return { url: `${url}?${params.toString()}`, body: undefined };
}

export function handleSaveImportTemplate(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { template } = payload as { template: Record<string, unknown> };
  return { url, body: JSON.stringify({ template }) };
}

export function handleLinkAccountTemplate(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { accountId, templateId, contextKind } = payload as {
    accountId: string;
    templateId: string;
    contextKind?: string;
  };
  return { url, body: JSON.stringify({ accountId, templateId, contextKind }) };
}
