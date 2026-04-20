// Health center domain handlers

export function handleDismissHealthIssue(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { issueId, dataHash } = payload as { issueId: string; dataHash: string };
  return { url, body: JSON.stringify({ issueId, dataHash }) };
}

export function handleRestoreHealthIssue(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { issueId } = payload as { issueId: string };
  return { url, body: JSON.stringify({ issueId }) };
}

export function handleExecuteHealthFix(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { action } = payload as { action: Record<string, unknown> };
  return { url, body: JSON.stringify(action) };
}

export function handleUpdateHealthConfig(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { config } = payload as { config: Record<string, unknown> };
  return { url, body: JSON.stringify(config) };
}
