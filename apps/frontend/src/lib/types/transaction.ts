// Transaction-domain TypeScript interfaces (Phase 4, plan 04-05).
//
// Money fields are JSON numbers (rust_decimal `serde-float` — Phase 3 fix 7e9eb697).
// Dates arrive as strings (ISO format); hooks normalize if needed.

// ---------------------------------------------------------------------------
// Enum-like string unions
// ---------------------------------------------------------------------------

export type TransactionDirection = "INCOME" | "EXPENSE" | "TRANSFER";
export type TransactionSource = "MANUAL" | "CSV" | "OFX" | "SYSTEM";
export type FxRateSource = "SYSTEM" | "MANUAL_OVERRIDE";
export type TransferLegRole = "SOURCE" | "DESTINATION";
export type CategorySource = "USER" | "MEMORY" | "IMPORT";
export type TransferEditMode = "APPLY_BOTH" | "THIS_LEG_ONLY";
export type DuplicateBucket = "ALMOST_CERTAIN" | "LIKELY" | "POSSIBLE";

// ---------------------------------------------------------------------------
// Core transaction types
// ---------------------------------------------------------------------------

export interface TransactionSplit {
  id: string;
  transactionId: string;
  categoryId: string;
  amount: number;
  notes: string | null;
  sortOrder: number;
}

export interface NewSplit {
  categoryId: string;
  amount: number;
  notes?: string | null;
  sortOrder: number;
}

export interface Transaction {
  id: string;
  accountId: string;
  direction: TransactionDirection;
  amount: number;
  currency: string;
  transactionDate: string; // YYYY-MM-DD
  payee: string | null;
  notes: string | null;
  categoryId: string | null;
  hasSplits: boolean;
  fxRate: number | null;
  fxRateSource: FxRateSource | null;
  transferGroupId: string | null;
  counterpartyAccountId: string | null;
  transferLegRole: TransferLegRole | null;
  idempotencyKey: string | null;
  importRunId: string | null;
  source: TransactionSource;
  externalRef: string | null;
  isSystemGenerated: boolean;
  isUserModified: boolean;
  categorySource: CategorySource | null;
  createdAt: string;
  updatedAt: string;
  splits: TransactionSplit[];
  isFxStale?: boolean;
}

export interface NewTransaction {
  accountId: string;
  direction: TransactionDirection;
  amount: number;
  currency: string;
  transactionDate: string;
  payee?: string | null;
  notes?: string | null;
  categoryId?: string | null;
  hasSplits?: boolean;
  fxRate?: number | null;
  fxRateSource?: FxRateSource | null;
  transferGroupId?: string | null;
  counterpartyAccountId?: string | null;
  transferLegRole?: TransferLegRole | null;
  source: TransactionSource;
  externalRef?: string | null;
  splits?: NewSplit[];
}

export interface TransactionUpdate {
  id: string;
  direction?: TransactionDirection;
  amount?: number;
  currency?: string;
  transactionDate?: string;
  payee?: string | null;
  notes?: string | null;
  categoryId?: string | null;
  hasSplits?: boolean;
  fxRate?: number | null;
  fxRateSource?: FxRateSource | null;
  splits?: NewSplit[];
}

export interface NewTransferLeg {
  accountId: string;
  amount: number;
  currency: string;
  transactionDate: string;
  notes?: string | null;
  categoryId?: string | null;
}

export interface TransactionWithRunningBalance extends Transaction {
  runningBalance: number;
}

// ---------------------------------------------------------------------------
// Duplicate detection
// ---------------------------------------------------------------------------

export interface DuplicateCandidate {
  candidateRowIndex: number;
  existingTransactionId: string;
  confidence: number; // 0..100
  bucket: DuplicateBucket;
}

// Backward-compat alias (04-04 adapter uses DuplicateMatch)
export type DuplicateMatch = DuplicateCandidate;

// ---------------------------------------------------------------------------
// Filters, sorting, search
// ---------------------------------------------------------------------------

export interface TransactionFilters {
  accountIds?: string[];
  categoryIds?: string[];
  directions?: TransactionDirection[];
  amountMin?: number;
  amountMax?: number;
  dateFrom?: string;
  dateTo?: string;
  showTransfers?: boolean;
  source?: TransactionSource[];
}

export interface TransactionSort {
  field: "transactionDate" | "amount" | "payee" | "createdAt";
  direction: "asc" | "desc";
}

export interface TransactionSearchResult {
  items: Transaction[];
  total: number;
}

// ---------------------------------------------------------------------------
// Payee category memory (D-12/15)
// ---------------------------------------------------------------------------

export interface PayeeCategoryMemory {
  accountId: string;
  normalizedMerchant: string;
  categoryId: string;
  lastSeenAt: string;
  seenCount: number;
}

// ---------------------------------------------------------------------------
// CSV import
// ---------------------------------------------------------------------------

export interface CsvFieldMapping {
  dateColumn: string;
  amountColumn?: string | null;
  debitColumn?: string | null;
  creditColumn?: string | null;
  payeeColumn: string;
  categoryColumn?: string | null;
  notesColumn?: string | null;
  currencyColumn?: string | null;
  externalIdColumn?: string | null;
  dateFormat: string;
  decimalSeparator: string;
  thousandsSeparator?: string | null;
}

export interface TransactionCsvImportRequest {
  accountId: string;
  file: File | Blob;
  mapping: CsvFieldMapping;
  templateName?: string;
}

export interface TransactionOfxImportRequest {
  accountId: string;
  file: File | Blob;
}

export interface TransactionImportResult {
  importRunId: string;
  inserted: number;
  skippedDuplicates: number;
  errors: string[];
  pendingDuplicateCount: number;
  /** Order-preserving: insertedRowIds[i] is the inserted txn id for input row i;
   *  empty string sentinel for rows skipped due to idempotency/duplicate.
   *  Required by plan 04-09 to wire pending duplicate pairs. */
  insertedRowIds: string[];
}

// Backward-compat alias (04-04 had ImportResult without pendingDuplicateCount)
export type ImportResult = TransactionImportResult;

export interface TransactionImportPreview {
  rows: NewTransaction[];
  duplicates: DuplicateCandidate[];
  compileErrors: { rowIndex: number; message: string }[];
}

// ---------------------------------------------------------------------------
// Import templates (D-16/17/18)
// ---------------------------------------------------------------------------

export interface TransactionTemplate {
  id: string;
  name: string;
  mapping: CsvFieldMapping;
  headerSignature: string[];
  createdAt: string;
  updatedAt: string;
}

export interface SaveTransactionTemplateRequest {
  id?: string;
  name: string;
  mapping: CsvFieldMapping;
  headerSignature: string[];
}

// Backward-compat alias (04-04 adapter uses NewTransactionTemplate)
export type NewTransactionTemplate = SaveTransactionTemplateRequest;
