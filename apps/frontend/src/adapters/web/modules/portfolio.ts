// Portfolio, performance, valuations, allocations domain handlers

export function handleGetHistoricalValuations(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const p = payload as { accountId?: string; startDate?: string; endDate?: string };
  const params = new URLSearchParams();
  if (p?.accountId) params.set("accountId", p.accountId);
  if (p?.startDate) params.set("startDate", p.startDate);
  if (p?.endDate) params.set("endDate", p.endDate);
  const qs = params.toString();
  return { url: qs ? `${url}?${qs}` : url, body: undefined };
}

export function handleGetLatestValuations(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const p = payload as { accountIds?: string[] };
  const params = new URLSearchParams();
  if (Array.isArray(p?.accountIds)) {
    for (const id of p.accountIds) params.append("accountIds[]", id);
  }
  const qs = params.toString();
  return { url: qs ? `${url}?${qs}` : url, body: undefined };
}

export function handleGetPortfolioAllocations(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { accountId } = payload as { accountId: string };
  const params = new URLSearchParams();
  params.set("accountId", accountId);
  return { url: `${url}?${params.toString()}`, body: undefined };
}

export function handleCalculateAccountsSimplePerformance(
  url: string,
  payload?: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { accountIds } = (payload ?? {}) as { accountIds?: string[] };
  return { url, body: JSON.stringify({ accountIds }) };
}

export function handleCalculatePerformanceHistory(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { itemType, itemId, startDate, endDate, trackingMode } = payload as {
    itemType: string;
    itemId: string;
    startDate?: string;
    endDate?: string;
    trackingMode?: string;
  };
  return { url, body: JSON.stringify({ itemType, itemId, startDate, endDate, trackingMode }) };
}

export function handleCalculatePerformanceSummary(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { itemType, itemId, startDate, endDate, trackingMode } = payload as {
    itemType: string;
    itemId: string;
    startDate?: string;
    endDate?: string;
    trackingMode?: string;
  };
  return { url, body: JSON.stringify({ itemType, itemId, startDate, endDate, trackingMode }) };
}

export function handleGetIncomeSummary(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { accountId: incomeAccountId } = payload as { accountId?: string };
  if (incomeAccountId) {
    return { url: `${url}?accountId=${encodeURIComponent(incomeAccountId)}`, body: undefined };
  }
  return { url, body: undefined };
}

// Net Worth
export function handleGetNetWorth(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { date } = (payload ?? {}) as { date?: string };
  if (date) {
    const params = new URLSearchParams();
    params.set("date", date);
    return { url: `${url}?${params.toString()}`, body: undefined };
  }
  return { url, body: undefined };
}

export function handleGetNetWorthHistory(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { startDate, endDate } = payload as { startDate: string; endDate: string };
  const params = new URLSearchParams();
  params.set("startDate", startDate);
  params.set("endDate", endDate);
  return { url: `${url}?${params.toString()}`, body: undefined };
}
