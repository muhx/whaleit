// Transaction Commands (Phase 4, plan 04-04)
//
// Typed wrappers around invoke<T>(commandName, payload). Each function
// catches errors, logs them, and re-throws so callers see the original.
//
// Multipart-upload routes (CSV / OFX) use FormData via the web adapter's
// special path-only commands; the adapter module in
// `apps/frontend/src/adapters/web/modules/transactions.ts` builds the
// FormData on the web side. On the desktop side, Tauri talks HTTP too
// (no IPC commands), so the same shared wrappers are sufficient.

import type {
  DuplicateMatch,
  NewTransaction,
  NewTransactionTemplate,
  NewTransferLeg,
  PayeeCategoryMemory,
  Transaction,
  TransactionFilters,
  TransactionSearchResult,
  TransactionTemplate,
  TransactionUpdate,
  TransactionWithRunningBalance,
  TransferEditMode,
} from "@/lib/types/transaction";

import { invoke, logger } from "./platform";

// ---------------------------------------------------------------------------
// CRUD
// ---------------------------------------------------------------------------

export const searchTransactions = async (
  filters: TransactionFilters,
  page: number,
  pageSize: number,
): Promise<TransactionSearchResult> => {
  try {
    return await invoke<TransactionSearchResult>("search_transactions", {
      filters,
      page,
      pageSize,
    });
  } catch (err) {
    logger.error("Error searching transactions.");
    throw err;
  }
};

export const getTransaction = async (id: string): Promise<Transaction> => {
  try {
    return await invoke<Transaction>("get_transaction", { id });
  } catch (err) {
    logger.error("Error fetching transaction.");
    throw err;
  }
};

export const createTransaction = async (transaction: NewTransaction): Promise<Transaction> => {
  try {
    return await invoke<Transaction>("create_transaction", { transaction });
  } catch (err) {
    logger.error("Error creating transaction.");
    throw err;
  }
};

export const updateTransaction = async (
  transaction: TransactionUpdate,
  editMode?: TransferEditMode,
): Promise<Transaction> => {
  try {
    return await invoke<Transaction>("update_transaction", {
      transaction,
      editMode,
    });
  } catch (err) {
    logger.error("Error updating transaction.");
    throw err;
  }
};

export const deleteTransaction = async (id: string): Promise<Transaction> => {
  try {
    return await invoke<Transaction>("delete_transaction", { id });
  } catch (err) {
    logger.error("Error deleting transaction.");
    throw err;
  }
};

// ---------------------------------------------------------------------------
// Recent + running balance
// ---------------------------------------------------------------------------

export const listRunningBalance = async (
  accountId: string,
  from?: string,
  to?: string,
): Promise<TransactionWithRunningBalance[]> => {
  try {
    return await invoke<TransactionWithRunningBalance[]>("list_running_balance", {
      accountId,
      from,
      to,
    });
  } catch (err) {
    logger.error("Error listing running balance.");
    throw err;
  }
};

export const getAccountRecentTransactions = async (
  accountId: string,
  limit?: number,
): Promise<Transaction[]> => {
  try {
    return await invoke<Transaction[]>("get_account_recent_transactions", {
      accountId,
      limit,
    });
  } catch (err) {
    logger.error("Error fetching recent transactions.");
    throw err;
  }
};

// ---------------------------------------------------------------------------
// Import (CSV + OFX use multipart — implemented in platform-specific
// `web/transactions.ts` and `tauri/transactions.ts`. Plan 04-05 wires those.)
// ---------------------------------------------------------------------------

export const previewTransactionImport = async (
  candidates: NewTransaction[],
): Promise<DuplicateMatch[]> => {
  try {
    return await invoke<DuplicateMatch[]>("preview_transaction_import", {
      candidates,
    });
  } catch (err) {
    logger.error("Error previewing transaction import.");
    throw err;
  }
};

export const detectTransactionDuplicates = async (
  candidates: NewTransaction[],
): Promise<DuplicateMatch[]> => {
  try {
    return await invoke<DuplicateMatch[]>("detect_transaction_duplicates", {
      candidates,
    });
  } catch (err) {
    logger.error("Error detecting transaction duplicates.");
    throw err;
  }
};

// ---------------------------------------------------------------------------
// Templates (D-16/17/18)
// ---------------------------------------------------------------------------

export const listTransactionTemplates = async (): Promise<TransactionTemplate[]> => {
  try {
    return await invoke<TransactionTemplate[]>("list_transaction_templates");
  } catch (err) {
    logger.error("Error listing transaction templates.");
    throw err;
  }
};

export const saveTransactionTemplate = async (
  template: NewTransactionTemplate,
): Promise<TransactionTemplate> => {
  try {
    return await invoke<TransactionTemplate>("save_transaction_template", { template });
  } catch (err) {
    logger.error("Error saving transaction template.");
    throw err;
  }
};

export const deleteTransactionTemplate = async (id: string): Promise<void> => {
  try {
    await invoke<void>("delete_transaction_template", { id });
  } catch (err) {
    logger.error("Error deleting transaction template.");
    throw err;
  }
};

export const getTransactionTemplate = async (id: string): Promise<TransactionTemplate> => {
  try {
    return await invoke<TransactionTemplate>("get_transaction_template", { id });
  } catch (err) {
    logger.error("Error fetching transaction template.");
    throw err;
  }
};

// ---------------------------------------------------------------------------
// Transfers
// ---------------------------------------------------------------------------

export const createTransfer = async (
  src: NewTransferLeg,
  dst: NewTransferLeg,
): Promise<[Transaction, Transaction]> => {
  try {
    return await invoke<[Transaction, Transaction]>("create_transfer", { src, dst });
  } catch (err) {
    logger.error("Error creating transfer.");
    throw err;
  }
};

export const updateTransferLeg = async (
  transaction: TransactionUpdate,
  editMode: TransferEditMode,
): Promise<Transaction> => {
  try {
    return await invoke<Transaction>("update_transfer_leg", { transaction, editMode });
  } catch (err) {
    logger.error("Error updating transfer leg.");
    throw err;
  }
};

export const breakTransferPair = async (legId: string): Promise<Transaction> => {
  try {
    return await invoke<Transaction>("break_transfer_pair", { legId });
  } catch (err) {
    logger.error("Error breaking transfer pair.");
    throw err;
  }
};

// ---------------------------------------------------------------------------
// Payee category memory
// ---------------------------------------------------------------------------

export const lookupPayeeCategory = async (
  accountId: string,
  payee: string,
): Promise<PayeeCategoryMemory | null> => {
  try {
    const result = await invoke<PayeeCategoryMemory | null>("lookup_payee_category", {
      accountId,
      payee,
    });
    return result ?? null;
  } catch (err) {
    logger.error("Error looking up payee category.");
    throw err;
  }
};

export const listPayeeCategoryMemory = async (
  accountId: string,
): Promise<PayeeCategoryMemory[]> => {
  try {
    return await invoke<PayeeCategoryMemory[]>("list_payee_category_memory", {
      accountId,
    });
  } catch (err) {
    logger.error("Error listing payee category memory.");
    throw err;
  }
};
