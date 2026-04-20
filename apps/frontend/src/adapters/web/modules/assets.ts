// Assets domain handlers

export function handleCreateAsset(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { payload: assetPayload } = payload as { payload: Record<string, unknown> };
  return { url, body: JSON.stringify(assetPayload) };
}

export function handleDeleteAsset(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { id } = payload as { id: string };
  return { url: `${url}/${encodeURIComponent(id)}`, body: undefined };
}

export function handleGetAssetProfile(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { assetId } = payload as { assetId: string };
  const params = new URLSearchParams();
  params.set("assetId", assetId);
  return { url: `${url}?${params.toString()}`, body: undefined };
}

export function handleUpdateAssetProfile(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { id, payload: bodyPayload } = payload as { id: string; payload: Record<string, unknown> };
  return { url: `${url}/${encodeURIComponent(id)}`, body: JSON.stringify(bodyPayload) };
}

export function handleUpdateQuoteMode(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { id, quoteMode } = payload as { id: string; quoteMode: string };
  return { url: `${url}/${encodeURIComponent(id)}`, body: JSON.stringify({ quoteMode }) };
}

// Contribution limits
export function handleCreateContributionLimit(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { newLimit } = payload as { newLimit: Record<string, unknown> };
  return { url, body: JSON.stringify(newLimit) };
}

export function handleUpdateContributionLimit(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { id, updatedLimit } = payload as { id: string; updatedLimit: Record<string, unknown> };
  return { url: `${url}/${encodeURIComponent(id)}`, body: JSON.stringify(updatedLimit) };
}

export function handleDeleteContributionLimit(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { id } = payload as { id: string };
  return { url: `${url}/${encodeURIComponent(id)}`, body: undefined };
}

export function handleCalculateDepositsForContributionLimit(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { limitId } = payload as { limitId: string };
  return { url: `${url}/${encodeURIComponent(limitId)}/deposits`, body: undefined };
}
