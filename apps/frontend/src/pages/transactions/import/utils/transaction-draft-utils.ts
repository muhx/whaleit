// Transaction draft utilities (Phase 4, plan 04-08).
//
// Forked from apps/frontend/src/pages/activity/import/utils/draft-utils.ts.
// Creates DraftTransaction[] from raw CSV rows + a CsvFieldMapping.
// No asset/symbol/holdings logic — transaction field set only.

import type { CsvFieldMapping, DuplicateCandidate } from "@/lib/types/transaction";
import type { DraftTransaction } from "../context/transaction-import-context";

// ─────────────────────────────────────────────────────────────────────────────
// Column index helpers
// ─────────────────────────────────────────────────────────────────────────────

function colIndex(headers: string[], col: string | null | undefined): number {
  if (!col) return -1;
  return headers.indexOf(col);
}

function cellAt(row: string[], idx: number): string {
  if (idx < 0 || idx >= row.length) return "";
  return (row[idx] ?? "").trim();
}

// ─────────────────────────────────────────────────────────────────────────────
// Amount parsing
// ─────────────────────────────────────────────────────────────────────────────

function parseAmount(
  raw: string,
  decimalSep: string,
  thousandsSep: string | null | undefined,
): number | null {
  if (!raw) return null;
  let s = raw;
  // Strip thousands separator if known
  if (thousandsSep && thousandsSep !== decimalSep) {
    s = s.split(thousandsSep).join("");
  }
  // Normalize decimal separator to "."
  if (decimalSep === ",") {
    s = s.replace(",", ".");
  }
  // Strip currency symbols, whitespace
  s = s.replace(/[^0-9.\-+]/g, "");
  const n = parseFloat(s);
  return isNaN(n) ? null : n;
}

// ─────────────────────────────────────────────────────────────────────────────
// Direction inference
// ─────────────────────────────────────────────────────────────────────────────

function inferDirection(amount: number): "INCOME" | "EXPENSE" | "TRANSFER" {
  if (amount > 0) return "INCOME";
  if (amount < 0) return "EXPENSE";
  return "EXPENSE"; // zero-amount defaults to expense
}

// ─────────────────────────────────────────────────────────────────────────────
// Date normalization (light — server re-parses authoritatively)
// ─────────────────────────────────────────────────────────────────────────────

function normalizeDate(raw: string, _dateFormat: string): string {
  // Return as-is for now; server re-parses. Just trim.
  return raw.trim();
}

// ─────────────────────────────────────────────────────────────────────────────
// Public API
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Build a DraftTransaction[] preview from raw CSV rows + mapping.
 * The server re-parses authoritatively on submit — this is for UI display only.
 * Cap to first 500 rows to guard against T-04-29 (DoS via huge CSV).
 */
export function createDraftTransactions(
  csvRows: string[][],
  headers: string[],
  mapping: CsvFieldMapping,
  accountId: string,
  accountCurrency: string,
): DraftTransaction[] {
  const MAX_PREVIEW_ROWS = 500;
  const rows = csvRows.slice(0, MAX_PREVIEW_ROWS);

  const dateIdx = colIndex(headers, mapping.dateColumn);
  const amountIdx = colIndex(headers, mapping.amountColumn ?? null);
  const debitIdx = colIndex(headers, mapping.debitColumn ?? null);
  const creditIdx = colIndex(headers, mapping.creditColumn ?? null);
  const payeeIdx = colIndex(headers, mapping.payeeColumn);
  const categoryIdx = colIndex(headers, mapping.categoryColumn ?? null);
  const notesIdx = colIndex(headers, mapping.notesColumn ?? null);
  const currencyIdx = colIndex(headers, mapping.currencyColumn ?? null);
  const externalIdx = colIndex(headers, mapping.externalIdColumn ?? null);

  return rows.map((row, i): DraftTransaction => {
    const rawDate = cellAt(row, dateIdx);
    const rawCurrency = cellAt(row, currencyIdx);
    const currency = rawCurrency || accountCurrency || "USD";

    // Amount: prefer single-column; fall back to debit/credit columns
    let amount = 0;
    if (amountIdx >= 0) {
      amount =
        parseAmount(cellAt(row, amountIdx), mapping.decimalSeparator, mapping.thousandsSeparator) ??
        0;
    } else if (debitIdx >= 0 || creditIdx >= 0) {
      const debit =
        parseAmount(cellAt(row, debitIdx), mapping.decimalSeparator, mapping.thousandsSeparator) ??
        0;
      const credit =
        parseAmount(cellAt(row, creditIdx), mapping.decimalSeparator, mapping.thousandsSeparator) ??
        0;
      // Debit = outflow (negative), credit = inflow (positive)
      amount = credit - debit;
    }

    const direction = inferDirection(amount);
    const absAmount = Math.abs(amount);

    const transactionDate = normalizeDate(rawDate, mapping.dateFormat);
    const payee = cellAt(row, payeeIdx) || null;
    const notes = cellAt(row, notesIdx) || null;
    const categoryId = cellAt(row, categoryIdx) || null;
    const externalRef = cellAt(row, externalIdx) || null;

    const errors = validateDraftRow({ rawDate, transactionDate, absAmount, mapping });

    return {
      rowIndex: i,
      accountId,
      direction,
      amount: absAmount,
      currency,
      transactionDate,
      payee,
      notes,
      categoryId,
      externalRef,
      isValid: errors.length === 0,
      validationErrors: errors,
      skip: false,
      duplicateBucket: null,
      duplicateConfidence: null,
      existingTransactionId: null,
      userResolution: null,
    };
  });
}

interface ValidateRowInput {
  rawDate: string;
  transactionDate: string;
  absAmount: number;
  mapping: CsvFieldMapping;
}

function validateDraftRow({
  rawDate,
  transactionDate,
  absAmount,
  mapping,
}: ValidateRowInput): string[] {
  const errors: string[] = [];
  if (!transactionDate || !rawDate) {
    errors.push("Date is required");
  }
  // Only validate amount/debit/credit completeness if columns are configured
  const hasAmountCol = !!mapping.amountColumn;
  const hasDebitCreditCols = !!mapping.debitColumn || !!mapping.creditColumn;
  if (!hasAmountCol && !hasDebitCreditCols) {
    errors.push("Amount column is required");
  }
  if (!mapping.payeeColumn) {
    errors.push("Payee column is required");
  }
  if (absAmount < 0) {
    errors.push("Amount must be non-negative");
  }
  return errors;
}

/**
 * Validate a single draft row; returns error strings.
 */
export function validateDraft(draft: DraftTransaction): string[] {
  const errors: string[] = [];
  if (!draft.transactionDate) errors.push("Date is required");
  if (draft.amount < 0) errors.push("Amount must be non-negative");
  return errors;
}

/**
 * Merge duplicate-detection results back onto draft rows.
 * Rows whose rowIndex appears in candidates get their bucket/confidence set.
 */
export function applyDuplicateMatches(
  drafts: DraftTransaction[],
  candidates: DuplicateCandidate[],
): DraftTransaction[] {
  const byRowIndex = new Map<number, DuplicateCandidate>();
  for (const c of candidates) {
    byRowIndex.set(c.candidateRowIndex, c);
  }

  return drafts.map((d): DraftTransaction => {
    const match = byRowIndex.get(d.rowIndex);
    if (!match) return d;
    return {
      ...d,
      duplicateBucket: match.bucket,
      duplicateConfidence: match.confidence,
      existingTransactionId: match.existingTransactionId,
    };
  });
}
