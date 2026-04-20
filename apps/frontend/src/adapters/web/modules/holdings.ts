// Holdings and snapshots domain handlers

export function handleGetHoldings(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const p = payload as { accountId: string };
  return { url: `${url}?accountId=${encodeURIComponent(p.accountId)}`, body: undefined };
}

export function handleGetHolding(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { accountId, assetId } = payload as { accountId: string; assetId: string };
  const params = new URLSearchParams();
  params.set("accountId", accountId);
  params.set("assetId", assetId);
  return { url: `${url}?${params.toString()}`, body: undefined };
}

export function handleGetAssetHoldings(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const p = payload as { assetId: string };
  return { url: `${url}?assetId=${encodeURIComponent(p.assetId)}`, body: undefined };
}

export function handleGetSnapshots(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { accountId, dateFrom, dateTo } = payload as {
    accountId: string;
    dateFrom?: string;
    dateTo?: string;
  };
  const params = new URLSearchParams();
  params.set("accountId", accountId);
  if (dateFrom) params.set("dateFrom", dateFrom);
  if (dateTo) params.set("dateTo", dateTo);
  return { url: `${url}?${params.toString()}`, body: undefined };
}

export function handleGetSnapshotByDate(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { accountId, date } = payload as { accountId: string; date: string };
  const params = new URLSearchParams();
  params.set("accountId", accountId);
  params.set("date", date);
  return { url: `${url}?${params.toString()}`, body: undefined };
}

export function handleDeleteSnapshot(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { accountId, date } = payload as { accountId: string; date: string };
  const params = new URLSearchParams();
  params.set("accountId", accountId);
  params.set("date", date);
  return { url: `${url}?${params.toString()}`, body: undefined };
}

export function handleSaveManualHoldings(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { accountId, holdings, cashBalances, snapshotDate } = payload as {
    accountId: string;
    holdings: unknown[];
    cashBalances: Record<string, string>;
    snapshotDate?: string;
  };
  return { url, body: JSON.stringify({ accountId, holdings, cashBalances, snapshotDate }) };
}

export function handleImportHoldingsCsv(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { accountId, snapshots } = payload as { accountId: string; snapshots: unknown[] };
  return { url, body: JSON.stringify({ accountId, snapshots }) };
}

export function handleCheckHoldingsImport(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { accountId, snapshots } = payload as { accountId: string; snapshots: unknown[] };
  return { url, body: JSON.stringify({ accountId, snapshots }) };
}
