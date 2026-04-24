// Alternative assets domain handlers

export function handleCreateAlternativeAsset(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { request } = payload as { request: Record<string, unknown> };
  return { url, body: JSON.stringify(request) };
}

export function handleUpdateAlternativeAssetValuation(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { assetId, request } = payload as { assetId: string; request: Record<string, unknown> };
  return { url: `${url}/${encodeURIComponent(assetId)}/valuation`, body: JSON.stringify(request) };
}

export function handleDeleteAlternativeAsset(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { assetId } = payload as { assetId: string };
  return { url: `${url}/${encodeURIComponent(assetId)}`, body: undefined };
}

export function handleLinkLiability(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { liabilityId, request } = payload as {
    liabilityId: string;
    request: Record<string, unknown>;
  };
  return { url: `${url}/${encodeURIComponent(liabilityId)}/link`, body: JSON.stringify(request) };
}

export function handleUnlinkLiability(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { liabilityId } = payload as { liabilityId: string };
  return { url: `${url}/${encodeURIComponent(liabilityId)}/unlink`, body: undefined };
}

export function handleUpdateAlternativeAssetMetadata(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { assetId, metadata, name, notes } = payload as {
    assetId: string;
    metadata: Record<string, string>;
    name?: string;
    notes?: string | null;
  };
  return {
    url: `${url}/${encodeURIComponent(assetId)}/metadata`,
    body: JSON.stringify({ metadata, name, notes }),
  };
}
