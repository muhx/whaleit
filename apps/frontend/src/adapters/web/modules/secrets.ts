// Secrets domain handlers

export function handleSetSecret(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { secretKey, secret } = payload as { secretKey: string; secret: string };
  return { url, body: JSON.stringify({ secretKey, secret }) };
}

export function handleGetSecret(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { secretKey } = payload as { secretKey: string };
  const params = new URLSearchParams();
  params.set("secretKey", secretKey);
  return { url: `${url}?${params.toString()}`, body: undefined };
}

export function handleDeleteSecret(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { secretKey } = payload as { secretKey: string };
  const params = new URLSearchParams();
  params.set("secretKey", secretKey);
  return { url: `${url}?${params.toString()}`, body: undefined };
}
