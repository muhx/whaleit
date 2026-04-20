// Exchange rates domain handlers

export function handleUpdateExchangeRate(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { rate } = payload as { rate: Record<string, unknown> };
  return { url, body: JSON.stringify(rate) };
}

export function handleAddExchangeRate(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { newRate } = payload as { newRate: Record<string, unknown> };
  return { url, body: JSON.stringify(newRate) };
}

export function handleDeleteExchangeRate(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { rateId } = payload as { rateId: string };
  return { url: `${url}/${encodeURIComponent(rateId)}`, body: undefined };
}
