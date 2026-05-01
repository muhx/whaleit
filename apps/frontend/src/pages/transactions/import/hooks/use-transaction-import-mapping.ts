// Transaction import mapping hook (Phase 4, plan 04-08).
//
// Forked from apps/frontend/src/pages/activity/import/hooks/use-import-mapping.ts.
// Provides mapping helpers for the transaction wizard — no asset-resolution logic.
// Includes validateHeaderSignature for D-17 (saved template header check).

import { useCallback } from "react";
import type { CsvFieldMapping, TransactionTemplate } from "@/lib/types/transaction";
import { useTransactionImport } from "../context/transaction-import-context";

// ─────────────────────────────────────────────────────────────────────────────
// D-17: Header signature validation
// ─────────────────────────────────────────────────────────────────────────────

export interface HeaderSignatureResult {
  matches: boolean;
  mismatchedColumns: string[];
}

/**
 * Compare a saved template's headerSignature against the file's current headers.
 * Returns { matches: true } if they align at every position.
 * Returns { matches: false, mismatchedColumns } with human-readable diff entries.
 */
export function validateHeaderSignature(
  template: TransactionTemplate,
  currentHeaders: string[],
): HeaderSignatureResult {
  const expected = template.headerSignature;
  const mismatched: string[] = [];
  for (let i = 0; i < expected.length; i++) {
    if (currentHeaders[i] !== expected[i]) {
      mismatched.push(`position ${i}: expected '${expected[i]}', got '${currentHeaders[i] ?? ""}'`);
    }
  }
  return { matches: mismatched.length === 0, mismatchedColumns: mismatched };
}

// ─────────────────────────────────────────────────────────────────────────────
// Auto-mapping heuristic
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Attempt to auto-map CSV headers to CsvFieldMapping fields based on common
 * column name patterns. Returns partial overrides; caller merges with defaults.
 */
export function computeTransactionFieldMappings(
  headers: string[],
  existing?: Partial<CsvFieldMapping>,
): Partial<CsvFieldMapping> {
  const lower = headers.map((h) => h.toLowerCase().trim());

  function firstMatch(...patterns: RegExp[]): string | null {
    for (const pat of patterns) {
      const idx = lower.findIndex((h) => pat.test(h));
      if (idx >= 0) return headers[idx];
    }
    return null;
  }

  const dateCol =
    existing?.dateColumn ||
    firstMatch(/\bdate\b/, /\btrans.*date\b/, /\bposted\b/, /\bvalue.*date\b/) ||
    "";

  const amountCol =
    existing?.amountColumn ??
    firstMatch(/\bamount\b/, /\btrnamt\b/, /\bvalue\b/, /\bsum\b/) ??
    null;

  const debitCol =
    existing?.debitColumn ?? firstMatch(/\bdebit\b/, /\bwithdrawal\b/, /\bout\b/) ?? null;

  const creditCol =
    existing?.creditColumn ?? firstMatch(/\bcredit\b/, /\bdeposit\b/, /\bin\b/) ?? null;

  const payeeCol =
    existing?.payeeColumn ||
    firstMatch(/\bpayee\b/, /\bname\b/, /\bdescription\b/, /\bmemo\b/, /\bmerchant\b/) ||
    "";

  const categoryCol =
    existing?.categoryColumn ?? firstMatch(/\bcategor\b/, /\btype\b/, /\bgroup\b/) ?? null;

  const notesCol =
    existing?.notesColumn ?? firstMatch(/\bnotes?\b/, /\bmemo\b/, /\bcomment\b/) ?? null;

  const currencyCol = existing?.currencyColumn ?? firstMatch(/\bcurrenc\b/, /\bccy\b/) ?? null;

  const externalIdCol =
    existing?.externalIdColumn ??
    firstMatch(/\bfitid\b/, /\bexternal.*id\b/, /\btransaction.*id\b/, /\bref\b/) ??
    null;

  return {
    dateColumn: dateCol,
    amountColumn: amountCol,
    debitColumn: debitCol,
    creditColumn: creditCol,
    payeeColumn: payeeCol,
    categoryColumn: categoryCol,
    notesColumn: notesCol,
    currencyColumn: currencyCol,
    externalIdColumn: externalIdCol,
  };
}

// ─────────────────────────────────────────────────────────────────────────────
// Hook
// ─────────────────────────────────────────────────────────────────────────────

export function useTransactionImportMapping() {
  const { state, dispatch } = useTransactionImport();

  const setMapping = useCallback(
    (mapping: CsvFieldMapping) => {
      dispatch({ type: "SET_MAPPING", payload: mapping });
    },
    [dispatch],
  );

  const applyTemplate = useCallback(
    (template: TransactionTemplate) => {
      const result = validateHeaderSignature(template, state.csvHeaders);
      if (!result.matches) {
        // Set template but flag the mismatch so MappingStep can show banner (D-17)
        dispatch({
          type: "SET_TEMPLATE",
          payload: { id: template.id, name: template.name, mapping: template.mapping },
        });
        dispatch({
          type: "SET_HEADER_SIGNATURE_MISMATCH",
          payload: { mismatch: true, details: result.mismatchedColumns },
        });
      } else {
        dispatch({
          type: "SET_TEMPLATE",
          payload: { id: template.id, name: template.name, mapping: template.mapping },
        });
      }
    },
    [dispatch, state.csvHeaders],
  );

  const clearTemplate = useCallback(() => {
    dispatch({ type: "CLEAR_TEMPLATE" });
  }, [dispatch]);

  return {
    mapping: state.mapping,
    selectedTemplateId: state.selectedTemplateId,
    selectedTemplateName: state.selectedTemplateName,
    headerSignatureMismatch: state.headerSignatureMismatch,
    headerSignatureMismatchDetails: state.headerSignatureMismatchDetails,
    setMapping,
    applyTemplate,
    clearTemplate,
    validateHeaderSignature,
  };
}
