// Transactions domain handlers (Phase 4, plan 04-04)
//
// Each handler receives the base URL from COMMANDS and the payload object,
// and returns { url, body } per the web adapter dispatch contract.
// GET/DELETE handlers move payload fields into query params; POST/PUT
// handlers stringify the payload (or a subset) into the request body.

type HandleResult = { url: string; body: string | undefined };

// ---------------------------------------------------------------------------
// CRUD
// ---------------------------------------------------------------------------

export function handleSearchTransactions(
  url: string,
  payload: Record<string, unknown>,
): HandleResult {
  return { url, body: JSON.stringify(payload) };
}

export function handleGetTransaction(url: string, payload: Record<string, unknown>): HandleResult {
  const { id } = payload as { id: string };
  const params = new URLSearchParams();
  params.set("id", id);
  return { url: `${url}?${params.toString()}`, body: undefined };
}

export function handleCreateTransaction(
  url: string,
  payload: Record<string, unknown>,
): HandleResult {
  const { transaction } = payload as { transaction: Record<string, unknown> };
  return { url, body: JSON.stringify(transaction) };
}

export function handleUpdateTransaction(
  url: string,
  payload: Record<string, unknown>,
): HandleResult {
  return { url, body: JSON.stringify(payload) };
}

export function handleDeleteTransaction(
  url: string,
  payload: Record<string, unknown>,
): HandleResult {
  const { id } = payload as { id: string };
  const params = new URLSearchParams();
  params.set("id", id);
  return { url: `${url}?${params.toString()}`, body: undefined };
}

// ---------------------------------------------------------------------------
// Recent + running balance
// ---------------------------------------------------------------------------

export function handleListRunningBalance(
  url: string,
  payload: Record<string, unknown>,
): HandleResult {
  return { url, body: JSON.stringify(payload) };
}

export function handleGetAccountRecentTransactions(
  url: string,
  payload: Record<string, unknown>,
): HandleResult {
  const { accountId, limit } = payload as { accountId: string; limit?: number };
  const params = new URLSearchParams();
  params.set("accountId", accountId);
  if (limit !== undefined && limit !== null) params.set("limit", String(limit));
  return { url: `${url}?${params.toString()}`, body: undefined };
}

// ---------------------------------------------------------------------------
// Import (preview + duplicates use JSON; CSV/OFX multipart handled
// platform-side in plan 04-05)
// ---------------------------------------------------------------------------

export function handlePreviewTransactionImport(
  url: string,
  payload: Record<string, unknown>,
): HandleResult {
  return { url, body: JSON.stringify(payload) };
}

export function handleDetectTransactionDuplicates(
  url: string,
  payload: Record<string, unknown>,
): HandleResult {
  return { url, body: JSON.stringify(payload) };
}

// ---------------------------------------------------------------------------
// Templates (D-16/17/18)
// ---------------------------------------------------------------------------

export function handleSaveTransactionTemplate(
  url: string,
  payload: Record<string, unknown>,
): HandleResult {
  const { template } = payload as { template: Record<string, unknown> };
  return { url, body: JSON.stringify({ template }) };
}

export function handleDeleteTransactionTemplate(
  url: string,
  payload: Record<string, unknown>,
): HandleResult {
  const { id } = payload as { id: string };
  const params = new URLSearchParams();
  params.set("id", id);
  return { url: `${url}?${params.toString()}`, body: undefined };
}

export function handleGetTransactionTemplate(
  url: string,
  payload: Record<string, unknown>,
): HandleResult {
  const { id } = payload as { id: string };
  const params = new URLSearchParams();
  params.set("id", id);
  return { url: `${url}?${params.toString()}`, body: undefined };
}

// ---------------------------------------------------------------------------
// Transfers
// ---------------------------------------------------------------------------

export function handleCreateTransfer(url: string, payload: Record<string, unknown>): HandleResult {
  return { url, body: JSON.stringify(payload) };
}

export function handleUpdateTransferLeg(
  url: string,
  payload: Record<string, unknown>,
): HandleResult {
  return { url, body: JSON.stringify(payload) };
}

export function handleBreakTransferPair(
  url: string,
  payload: Record<string, unknown>,
): HandleResult {
  return { url, body: JSON.stringify(payload) };
}

// ---------------------------------------------------------------------------
// Payee category memory
// ---------------------------------------------------------------------------

export function handleLookupPayeeCategory(
  url: string,
  payload: Record<string, unknown>,
): HandleResult {
  return { url, body: JSON.stringify(payload) };
}

export function handleListPayeeCategoryMemory(
  url: string,
  payload: Record<string, unknown>,
): HandleResult {
  const { accountId } = payload as { accountId: string };
  const params = new URLSearchParams();
  params.set("accountId", accountId);
  return { url: `${url}?${params.toString()}`, body: undefined };
}
