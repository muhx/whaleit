import { keepPreviousData, useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  breakTransferPair,
  createTransaction,
  createTransfer,
  deleteTransaction,
  detectTransactionDuplicates,
  getAccountRecentTransactions,
  getTransaction,
  importTransactionsCsv,
  importTransactionsOfx,
  listRunningBalance,
  searchTransactions,
  updateTransaction,
  updateTransferLeg,
} from "@/adapters";
import { QueryKeys } from "@/lib/query-keys";
import type {
  DuplicateCandidate,
  NewTransaction,
  NewTransferLeg,
  Transaction,
  TransactionCsvImportRequest,
  TransactionFilters,
  TransactionImportResult,
  TransactionOfxImportRequest,
  TransactionSearchResult,
  TransactionSort,
  TransactionUpdate,
  TransactionWithRunningBalance,
  TransferEditMode,
} from "@/lib/types/transaction";

// ---------------------------------------------------------------------------
// Queries
// ---------------------------------------------------------------------------

export function useTransactionSearch(
  filters: TransactionFilters = {},
  page = 0,
  pageSize = 50,
  searchKeyword = "",
  sort?: TransactionSort,
) {
  return useQuery<TransactionSearchResult>({
    queryKey: QueryKeys.TRANSACTIONS_SEARCH(filters, page, pageSize, searchKeyword),
    queryFn: () => searchTransactions(page, pageSize, filters, searchKeyword, sort),
    placeholderData: keepPreviousData,
  });
}

export function useTransaction(id: string | null) {
  return useQuery<Transaction>({
    queryKey: ["transactions", "item", id],
    queryFn: () => getTransaction(id!),
    enabled: !!id,
  });
}

export function useRunningBalance(accountId: string | null, from?: string, to?: string) {
  return useQuery<TransactionWithRunningBalance[]>({
    queryKey: QueryKeys.RUNNING_BALANCE(accountId ?? "", from, to),
    queryFn: () => listRunningBalance(accountId!, from, to),
    enabled: !!accountId,
  });
}

export function useAccountRecentTransactions(accountId: string | null, limit = 10) {
  return useQuery<Transaction[]>({
    queryKey: QueryKeys.ACCOUNT_RECENT_TRANSACTIONS(accountId ?? "", limit),
    queryFn: () => getAccountRecentTransactions(accountId!, limit),
    enabled: !!accountId,
  });
}

// ---------------------------------------------------------------------------
// Shared invalidation helper
// ---------------------------------------------------------------------------

function invalidateTransactionLists(qc: ReturnType<typeof useQueryClient>) {
  qc.invalidateQueries({ queryKey: QueryKeys.TRANSACTIONS });
  qc.invalidateQueries({ queryKey: ["transactions", "running-balance"] });
  qc.invalidateQueries({ queryKey: ["transactions", "by-account-recent"] });
  qc.invalidateQueries({ queryKey: [QueryKeys.ACCOUNTS] }); // current_balance derives from txns
}

// ---------------------------------------------------------------------------
// Mutations
// ---------------------------------------------------------------------------

export function useCreateTransaction() {
  const qc = useQueryClient();
  return useMutation<Transaction, Error, NewTransaction>({
    mutationFn: (txn) => createTransaction(txn),
    onSuccess: () => invalidateTransactionLists(qc),
  });
}

export function useUpdateTransaction() {
  const qc = useQueryClient();
  return useMutation<
    Transaction,
    Error,
    { transaction: TransactionUpdate; editMode?: TransferEditMode }
  >({
    mutationFn: ({ transaction, editMode }) => updateTransaction(transaction, editMode),
    onSuccess: (data) => {
      invalidateTransactionLists(qc);
      qc.invalidateQueries({ queryKey: ["transactions", "item", data.id] });
    },
  });
}

export function useDeleteTransaction() {
  const qc = useQueryClient();
  return useMutation<Transaction, Error, string>({
    mutationFn: (id) => deleteTransaction(id),
    onSuccess: () => invalidateTransactionLists(qc),
  });
}

export function useCreateTransfer() {
  const qc = useQueryClient();
  return useMutation<
    [Transaction, Transaction],
    Error,
    { src: NewTransferLeg; dst: NewTransferLeg }
  >({
    mutationFn: ({ src, dst }) => createTransfer(src, dst),
    onSuccess: () => invalidateTransactionLists(qc),
  });
}

export function useUpdateTransferLeg() {
  const qc = useQueryClient();
  return useMutation<
    Transaction,
    Error,
    { transaction: TransactionUpdate; editMode: TransferEditMode }
  >({
    mutationFn: ({ transaction, editMode }) => updateTransferLeg(transaction, editMode),
    onSuccess: () => invalidateTransactionLists(qc),
  });
}

export function useBreakTransferPair() {
  const qc = useQueryClient();
  return useMutation<Transaction, Error, string>({
    mutationFn: (legId) => breakTransferPair(legId),
    onSuccess: () => invalidateTransactionLists(qc),
  });
}

export function useImportTransactionsCsv() {
  const qc = useQueryClient();
  return useMutation<TransactionImportResult, Error, TransactionCsvImportRequest>({
    mutationFn: (req) => importTransactionsCsv(req),
    onSuccess: () => invalidateTransactionLists(qc),
  });
}

export function useImportTransactionsOfx() {
  const qc = useQueryClient();
  return useMutation<TransactionImportResult, Error, TransactionOfxImportRequest>({
    mutationFn: (req) => importTransactionsOfx(req),
    onSuccess: () => invalidateTransactionLists(qc),
  });
}

export function useDetectTransactionDuplicates() {
  return useMutation<DuplicateCandidate[], Error, NewTransaction[]>({
    mutationFn: (candidates) => detectTransactionDuplicates(candidates),
  });
}
