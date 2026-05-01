// Transaction-related type stubs (Phase 4, plan 04-04).
//
// Plan 04-05 will expand these with proper hooks/zod schemas. This stub
// exists so the adapter layer in plan 04-04 type-checks. The shapes mirror
// the Rust core types in `crates/core/src/transactions/transactions_model.rs`
// and `transactions_traits.rs`.
//
// Money fields are JSON numbers (rust_decimal `serde-float`).
// Dates from the wire arrive as strings; hooks in 04-05 may normalize to Date.

export interface CsvFieldMapping {
  dateColumn: string;
  amountColumn?: string;
  debitColumn?: string;
  creditColumn?: string;
  payeeColumn: string;
  categoryColumn?: string;
  notesColumn?: string;
  currencyColumn?: string;
  externalIdColumn?: string;
  dateFormat: string;
  decimalSeparator: string;
  thousandsSeparator?: string;
}

export interface NewSplit {
  categoryId: string;
  amount: number;
  notes?: string;
  sortOrder: number;
}

export interface TransactionSplit {
  id: string;
  transactionId: string;
  categoryId: string;
  amount: number;
  notes?: string;
  sortOrder: number;
  createdAt: string;
  updatedAt: string;
}

export interface Transaction {
  id: string;
  accountId: string;
  direction: string;
  amount: number;
  currency: string;
  transactionDate: string;
  payee?: string;
  notes?: string;
  categoryId?: string;
  hasSplits: boolean;
  fxRate?: number;
  fxRateSource?: string;
  transferGroupId?: string;
  counterpartyAccountId?: string;
  transferLegRole?: string;
  idempotencyKey?: string;
  importRunId?: string;
  source: string;
  externalRef?: string;
  isSystemGenerated: boolean;
  isUserModified: boolean;
  categorySource?: string;
  createdAt: string;
  updatedAt: string;
  splits: TransactionSplit[];
}

export interface NewTransaction {
  accountId: string;
  direction: string;
  amount: number;
  currency: string;
  transactionDate: string;
  payee?: string;
  notes?: string;
  categoryId?: string;
  hasSplits: boolean;
  fxRate?: number;
  fxRateSource?: string;
  transferGroupId?: string;
  counterpartyAccountId?: string;
  transferLegRole?: string;
  idempotencyKey?: string;
  importRunId?: string;
  source: string;
  externalRef?: string;
  isSystemGenerated: boolean;
  isUserModified: boolean;
  categorySource?: string;
  splits: NewSplit[];
}

export interface TransactionUpdate {
  id: string;
  direction?: string;
  amount?: number;
  currency?: string;
  transactionDate?: string;
  payee?: string;
  notes?: string;
  categoryId?: string;
  hasSplits?: boolean;
  fxRate?: number;
  fxRateSource?: string;
  transferGroupId?: string;
  counterpartyAccountId?: string;
  transferLegRole?: string;
  idempotencyKey?: string;
  importRunId?: string;
  source?: string;
  externalRef?: string;
  isSystemGenerated?: boolean;
  isUserModified?: boolean;
  categorySource?: string;
  splits?: NewSplit[];
}

export interface TransactionFilters {
  accountIds: string[];
  categoryIds: string[];
  directions: string[];
  amountMin?: number;
  amountMax?: number;
  dateFrom?: string;
  dateTo?: string;
  showTransfers: boolean;
  searchKeyword?: string;
}

export interface TransactionSearchResult {
  items: Transaction[];
  total: number;
}

export interface TransactionWithRunningBalance {
  txn: Transaction;
  runningBalance: number;
}

export interface NewTransferLeg {
  accountId: string;
  amount: number;
  currency: string;
  transactionDate: string;
  notes?: string;
  categoryId?: string;
  fxRate?: number;
  fxRateSource?: string;
}

export type TransferEditMode = "APPLY_BOTH" | "THIS_LEG_ONLY";

export interface PayeeCategoryMemory {
  accountId: string;
  normalizedMerchant: string;
  categoryId: string;
  lastSeenAt: string;
  seenCount: number;
}

export type DuplicateBucket = "ALMOST_CERTAIN" | "LIKELY" | "POSSIBLE";

export interface DuplicateMatch {
  candidateRowIndex: number;
  existingTransactionId: string;
  confidence: number;
  bucket: DuplicateBucket;
}

/**
 * Result of a CSV or OFX import operation.
 *
 * `insertedRowIds[i]` is the ID of the transaction inserted from input row i.
 * Empty string sentinel for rows skipped due to idempotency / duplicate.
 */
export interface ImportResult {
  importRunId: string;
  inserted: number;
  skippedDuplicates: number;
  errors: string[];
  insertedRowIds: string[];
  duplicateMatches: DuplicateMatch[];
}

export interface TransactionCsvImportRequest {
  accountId: string;
  accountCurrency: string;
  file: File | Blob;
  mapping: CsvFieldMapping;
  importRunId?: string;
}

export interface TransactionOfxImportRequest {
  accountId: string;
  accountCurrency: string;
  file: File | Blob;
  importRunId?: string;
}

export interface TransactionTemplate {
  id: string;
  name: string;
  mapping: CsvFieldMapping;
  headerSignature: string[];
  createdAt: string;
  updatedAt: string;
}

export interface NewTransactionTemplate {
  name: string;
  mapping: CsvFieldMapping;
  headerSignature: string[];
}
