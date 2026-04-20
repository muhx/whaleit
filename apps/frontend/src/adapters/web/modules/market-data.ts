// Market data domain handlers (quotes, symbols, providers, exchanges, custom providers)

export function handleSearchSymbol(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { query } = payload as { query: string };
  const params = new URLSearchParams();
  params.set("query", query);
  return { url: `${url}?${params.toString()}`, body: undefined };
}

export function handleResolveSymbolQuote(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { symbol, exchangeMic, instrumentType, providerId, quoteCcy } = payload as {
    symbol: string;
    exchangeMic?: string;
    instrumentType?: string;
    providerId?: string;
    quoteCcy?: string;
  };
  const params = new URLSearchParams();
  params.set("symbol", symbol);
  if (exchangeMic) params.set("exchangeMic", exchangeMic);
  if (instrumentType) params.set("instrumentType", instrumentType);
  if (providerId) params.set("providerId", providerId);
  if (quoteCcy) params.set("quoteCcy", quoteCcy);
  return { url: `${url}?${params.toString()}`, body: undefined };
}

export function handleGetQuoteHistory(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { symbol } = payload as { symbol: string };
  const params = new URLSearchParams();
  params.set("symbol", symbol);
  return { url: `${url}?${params.toString()}`, body: undefined };
}

export function handleGetLatestQuotes(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { assetIds } = payload as { assetIds: string[] };
  return { url, body: JSON.stringify({ assetIds }) };
}

export function handleUpdateQuote(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { symbol, quote } = payload as { symbol: string; quote: Record<string, unknown> };
  return { url: `${url}/${encodeURIComponent(symbol)}`, body: JSON.stringify(quote) };
}

export function handleDeleteQuote(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { id } = payload as { id: string };
  return { url: `${url}/${encodeURIComponent(id)}`, body: undefined };
}

export function handleCheckQuotesImport(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { content, hasHeaderRow } = payload as { content: number[]; hasHeaderRow: boolean };
  return { url, body: JSON.stringify({ content, hasHeaderRow }) };
}

export function handleImportQuotesCsv(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { quotes, overwriteExisting } = payload as { quotes: unknown; overwriteExisting: boolean };
  return { url, body: JSON.stringify({ quotes, overwriteExisting }) };
}

export function handleUpdateMarketDataProviderSettings(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  return { url, body: JSON.stringify(payload) };
}

export function handleSyncMarketData(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  return { url, body: JSON.stringify(payload) };
}

// Custom providers
export function handleCreateCustomProvider(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { payload: cp } = payload as { payload: Record<string, unknown> };
  return { url, body: JSON.stringify(cp) };
}

export function handleUpdateCustomProvider(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { providerId, payload: cp } = payload as {
    providerId: string;
    payload: Record<string, unknown>;
  };
  return { url: `${url}/${encodeURIComponent(providerId)}`, body: JSON.stringify(cp) };
}

export function handleDeleteCustomProvider(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { providerId } = payload as { providerId: string };
  return { url: `${url}/${encodeURIComponent(providerId)}`, body: undefined };
}

export function handleTestCustomProviderSource(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { payload: tp } = payload as { payload: Record<string, unknown> };
  return { url, body: JSON.stringify(tp) };
}
