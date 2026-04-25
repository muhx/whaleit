/**
 * Credit-card derivation helpers (Phase 3, D-08, D-13).
 * Decimal money values arrive as JSON numbers from the server (rust_decimal
 * serde-float).
 */

export type UtilizationTier = "success" | "warning" | "destructive";

/**
 * Returns the available credit as a number, or undefined when inputs are missing.
 * Per D-13, balance is stored positive; available = limit - balance.
 */
export function availableCredit(
  creditLimit: number | undefined,
  currentBalance: number | undefined,
): number | undefined {
  if (creditLimit === undefined || !Number.isFinite(creditLimit)) return undefined;
  const balance = currentBalance ?? 0;
  if (!Number.isFinite(balance)) return creditLimit;
  return Math.max(creditLimit - balance, 0);
}

/**
 * Returns utilization 0..100 as an integer, or undefined when inputs invalid.
 * Per D-08: utilization = current_balance / credit_limit * 100.
 */
export function utilizationPercent(
  creditLimit: number | undefined,
  currentBalance: number | undefined,
): number | undefined {
  if (creditLimit === undefined || !Number.isFinite(creditLimit) || creditLimit <= 0) {
    return undefined;
  }
  const balance = currentBalance ?? 0;
  if (!Number.isFinite(balance)) return 0;
  return Math.min(Math.round((balance / creditLimit) * 100), 100);
}

/**
 * Maps utilization percent to UI-SPEC color tier.
 * 0-29% -> success, 30-89% -> warning, 90-100%+ -> destructive.
 * Caller decides whether to render warning as tinted or solid (UI-SPEC §Color).
 */
export function utilizationTier(percent: number | undefined): UtilizationTier | undefined {
  if (percent === undefined) return undefined;
  if (percent >= 90) return "destructive";
  if (percent >= 30) return "warning";
  return "success";
}
