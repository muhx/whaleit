// Settings domain handlers

export function handleUpdateSettings(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const data = payload as { settingsUpdate: Record<string, unknown> };
  return { url, body: JSON.stringify(data.settingsUpdate) };
}

export function handleCheckUpdate(
  url: string,
  payload?: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { currentVersion, target, arch, force } = (payload ?? {}) as {
    currentVersion?: string;
    target?: string;
    arch?: string;
    force?: boolean;
  };
  const params = new URLSearchParams();
  if (currentVersion) params.set("currentVersion", currentVersion);
  if (target) params.set("target", target);
  if (arch) params.set("arch", arch);
  if (force) params.set("force", "true");
  const qs = params.toString();
  return { url: qs ? `${url}?${qs}` : url, body: undefined };
}
