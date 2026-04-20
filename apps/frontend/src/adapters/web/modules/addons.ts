// Addons domain handlers

import { toBase64 } from "../core";

export function handleInstallAddonZip(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { zipData, enableAfterInstall } = payload as {
    zipData: Uint8Array | number[];
    enableAfterInstall?: boolean;
  };
  const zipDataB64 = toBase64(zipData);
  return { url, body: JSON.stringify({ zipDataB64, enableAfterInstall }) };
}

export function handleToggleAddon(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { addonId, enabled } = payload as { addonId: string; enabled: boolean };
  return { url, body: JSON.stringify({ addonId, enabled }) };
}

export function handleUninstallAddon(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { addonId } = payload as { addonId: string };
  return { url: `${url}/${encodeURIComponent(addonId)}`, body: undefined };
}

export function handleLoadAddonForRuntime(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { addonId } = payload as { addonId: string };
  return { url: `${url}/${encodeURIComponent(addonId)}`, body: undefined };
}

export function handleExtractAddonZip(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { zipData } = payload as { zipData: Uint8Array | number[] };
  const zipDataB64 = toBase64(zipData);
  return { url, body: JSON.stringify({ zipDataB64 }) };
}

export function handleCheckAddonUpdate(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { addonId } = payload as { addonId: string };
  return { url, body: JSON.stringify({ addonId }) };
}

export function handleUpdateAddonFromStoreById(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { addonId } = payload as { addonId: string };
  return { url, body: JSON.stringify({ addonId }) };
}

export function handleDownloadAddonToStaging(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { addonId } = payload as { addonId: string };
  return { url, body: JSON.stringify({ addonId }) };
}

export function handleInstallAddonFromStaging(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { addonId, enableAfterInstall } = payload as { addonId: string; enableAfterInstall?: boolean };
  return { url, body: JSON.stringify({ addonId, enableAfterInstall }) };
}

export function handleClearAddonStaging(
  url: string,
  payload?: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { addonId } = (payload ?? {}) as { addonId?: string };
  if (addonId) {
    const params = new URLSearchParams();
    params.set("addonId", addonId);
    return { url: `${url}?${params.toString()}`, body: undefined };
  }
  return { url, body: undefined };
}

export function handleSubmitAddonRating(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { addonId, rating, review } = payload as { addonId: string; rating: number; review?: string };
  return { url, body: JSON.stringify({ addonId, rating, review }) };
}

export function handleGetAddonRatings(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { addonId } = payload as { addonId: string };
  const params = new URLSearchParams();
  params.set("addonId", addonId);
  return { url: `${url}?${params.toString()}`, body: undefined };
}
