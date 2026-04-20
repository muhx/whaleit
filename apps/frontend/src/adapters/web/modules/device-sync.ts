// Device sync domain handlers (pairing, E2EE, enrollment)

export function handleRegisterDevice(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { displayName, instanceId } = payload as { displayName: string; instanceId: string };
  // Detect platform from browser user agent
  const userAgent = navigator.userAgent.toLowerCase();
  let platform = "server"; // default fallback
  if (userAgent.includes("mac")) platform = "macos";
  else if (userAgent.includes("win")) platform = "windows";
  else if (userAgent.includes("linux") && !userAgent.includes("android")) platform = "linux";
  else if (userAgent.includes("android")) platform = "android";
  else if (userAgent.includes("iphone") || userAgent.includes("ipad")) platform = "ios";
  return { url, body: JSON.stringify({ displayName, platform, instanceId }) };
}

export function handleGetDevice(
  url: string,
  payload?: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { deviceId } = (payload ?? {}) as { deviceId?: string };
  if (deviceId) {
    return { url: `${url}/${encodeURIComponent(deviceId)}`, body: undefined };
  }
  return { url: `${url}/current`, body: undefined };
}

export function handleUpdateDevice(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { deviceId, displayName } = payload as { deviceId: string; displayName: string };
  return { url: `${url}/${encodeURIComponent(deviceId)}`, body: JSON.stringify({ displayName }) };
}

export function handleDeleteDevice(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { deviceId } = payload as { deviceId: string };
  return { url: `${url}/${encodeURIComponent(deviceId)}`, body: undefined };
}

export function handleRevokeDevice(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { deviceId } = payload as { deviceId: string };
  return { url: `${url}/${encodeURIComponent(deviceId)}/revoke`, body: undefined };
}

// Team keys (E2EE)
export function handleCommitInitializeTeamKeys(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { keyVersion, deviceKeyEnvelope, signature, challengeResponse, recoveryEnvelope } =
    payload as {
      keyVersion: number;
      deviceKeyEnvelope: string;
      signature: string;
      challengeResponse?: string;
      recoveryEnvelope?: string;
    };
  return {
    url,
    body: JSON.stringify({ keyVersion, deviceKeyEnvelope, signature, challengeResponse, recoveryEnvelope }),
  };
}

export function handleCommitRotateTeamKeys(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { newKeyVersion, envelopes, signature, challengeResponse } = payload as {
    newKeyVersion: number;
    envelopes: { deviceId: string; deviceKeyEnvelope: string }[];
    signature: string;
    challengeResponse?: string;
  };
  return { url, body: JSON.stringify({ newKeyVersion, envelopes, signature, challengeResponse }) };
}

export function handleResetTeamSync(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { reason } = (payload ?? {}) as { reason?: string };
  return { url, body: reason ? JSON.stringify({ reason }) : JSON.stringify({}) };
}

// Pairing (Issuer - Trusted Device)
export function handleCreatePairing(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { codeHash, ephemeralPublicKey } = payload as { codeHash: string; ephemeralPublicKey: string };
  return { url, body: JSON.stringify({ codeHash, ephemeralPublicKey }) };
}

export function handleGetPairing(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { pairingId } = payload as { pairingId: string };
  return { url: `${url}/${encodeURIComponent(pairingId)}`, body: undefined };
}

export function handleApprovePairing(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { pairingId } = payload as { pairingId: string };
  return { url: `${url}/${encodeURIComponent(pairingId)}/approve`, body: undefined };
}

export function handleCompletePairing(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { pairingId, encryptedKeyBundle, sasProof, signature } = payload as {
    pairingId: string;
    encryptedKeyBundle: string;
    sasProof: string | Record<string, unknown>;
    signature: string;
  };
  return { url: `${url}/${encodeURIComponent(pairingId)}/complete`, body: JSON.stringify({ encryptedKeyBundle, sasProof, signature }) };
}

export function handleCancelPairing(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { pairingId } = payload as { pairingId: string };
  return { url: `${url}/${encodeURIComponent(pairingId)}/cancel`, body: undefined };
}

// Claimer-side pairing commands
export function handleClaimPairing(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { code, ephemeralPublicKey } = payload as { code: string; ephemeralPublicKey: string };
  return { url, body: JSON.stringify({ code, ephemeralPublicKey }) };
}

export function handleGetPairingMessages(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { pairingId } = payload as { pairingId: string };
  return { url: `${url}/${encodeURIComponent(pairingId)}/messages`, body: undefined };
}

export function handleConfirmPairing(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  const { pairingId, proof, minSnapshotCreatedAt } = payload as {
    pairingId: string;
    proof?: string;
    minSnapshotCreatedAt?: string;
  };
  return { url: `${url}/${encodeURIComponent(pairingId)}/confirm`, body: JSON.stringify({ proof, minSnapshotCreatedAt }) };
}

export function handleCompletePairingWithTransfer(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  return { url, body: JSON.stringify(payload) };
}

export function handleConfirmPairingWithBootstrap(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  return { url, body: JSON.stringify(payload) };
}

export function handleBeginPairingConfirm(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  return { url, body: JSON.stringify(payload) };
}

export function handleGetPairingFlowState(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  return { url, body: JSON.stringify(payload) };
}

export function handleApprovePairingOverwrite(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  return { url, body: JSON.stringify(payload) };
}

export function handleCancelPairingFlow(
  url: string,
  payload: Record<string, unknown>,
): { url: string; body: string | undefined } {
  return { url, body: JSON.stringify(payload) };
}
