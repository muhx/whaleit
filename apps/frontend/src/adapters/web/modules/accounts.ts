// Accounts domain handlers

export function handleCreateAccount(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const data = payload as { account: Record<string, unknown> };
  return { url, body: JSON.stringify(data.account) };
}

export function handleUpdateAccount(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const data = payload as { accountUpdate: { id: string } & Record<string, unknown> };
  return {
    url: `${url}/${encodeURIComponent(data.accountUpdate.id)}`,
    body: JSON.stringify(data.accountUpdate),
  };
}

export function handleDeleteAccount(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const data = payload as { accountId: string };
  return { url: `${url}/${encodeURIComponent(data.accountId)}`, body: undefined };
}

export function handleGetAccounts(
  url: string,
  payload?: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { includeArchived } = (payload ?? {}) as { includeArchived?: boolean };
  if (includeArchived) {
    const params = new URLSearchParams();
    params.set("includeArchived", "true");
    return { url: `${url}?${params.toString()}`, body: undefined };
  }
  return { url, body: undefined };
}
