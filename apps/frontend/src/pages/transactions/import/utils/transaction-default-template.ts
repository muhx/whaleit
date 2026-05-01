// Default/empty CsvFieldMapping for the transaction import wizard (Phase 4, plan 04-08).
// Forked from apps/frontend/src/pages/activity/import/utils/default-activity-template.ts.

import type { CsvFieldMapping } from "@/lib/types/transaction";

export const DEFAULT_TRANSACTION_MAPPING: CsvFieldMapping = {
  dateColumn: "",
  amountColumn: null,
  debitColumn: null,
  creditColumn: null,
  payeeColumn: "",
  categoryColumn: null,
  notesColumn: null,
  currencyColumn: null,
  externalIdColumn: null,
  dateFormat: "auto",
  decimalSeparator: ".",
  thousandsSeparator: null,
};

export function createDefaultTransactionMapping(): CsvFieldMapping {
  return { ...DEFAULT_TRANSACTION_MAPPING };
}
