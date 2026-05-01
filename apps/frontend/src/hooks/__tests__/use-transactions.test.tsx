import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import type { Transaction, TransactionImportResult } from "@/lib/types/transaction";

// Hoisted mocks — vi.mock is hoisted so we use vi.hoisted for the mock fns.
const adapterMocks = vi.hoisted(() => ({
  searchTransactions: vi.fn(),
  getTransaction: vi.fn(),
  createTransaction: vi.fn(),
  updateTransaction: vi.fn(),
  deleteTransaction: vi.fn(),
  listRunningBalance: vi.fn(),
  getAccountRecentTransactions: vi.fn(),
  importTransactionsCsv: vi.fn(),
  importTransactionsOfx: vi.fn(),
  detectTransactionDuplicates: vi.fn(),
  createTransfer: vi.fn(),
  updateTransferLeg: vi.fn(),
  breakTransferPair: vi.fn(),
  lookupPayeeCategory: vi.fn(),
  listPayeeCategoryMemory: vi.fn(),
}));

vi.mock("@/adapters", () => adapterMocks);

import {
  useCreateTransaction,
  useCreateTransfer,
  useDeleteTransaction,
  useImportTransactionsCsv,
  useTransactionSearch,
  useUpdateTransaction,
} from "../use-transactions";
import { useLookupPayeeCategory } from "../use-merchant-categories";

const sampleTransaction: Transaction = {
  id: "t1",
  accountId: "a1",
  direction: "EXPENSE",
  amount: 10,
  currency: "USD",
  transactionDate: "2026-04-30",
  payee: "Coffee shop",
  notes: null,
  categoryId: "cat-1",
  hasSplits: false,
  fxRate: null,
  fxRateSource: null,
  transferGroupId: null,
  counterpartyAccountId: null,
  transferLegRole: null,
  idempotencyKey: null,
  importRunId: null,
  source: "MANUAL",
  externalRef: null,
  isSystemGenerated: false,
  isUserModified: false,
  categorySource: null,
  createdAt: "2026-04-30T00:00:00Z",
  updatedAt: "2026-04-30T00:00:00Z",
  splits: [],
};

function makeQueryClient() {
  return new QueryClient({
    defaultOptions: {
      queries: { retry: false, gcTime: 0, staleTime: 0 },
      mutations: { retry: false },
    },
  });
}

function makeWrapper(qc: QueryClient) {
  return ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={qc}>{children}</QueryClientProvider>
  );
}

describe("use-transactions hooks", () => {
  let qc: QueryClient;

  beforeEach(() => {
    vi.clearAllMocks();
    qc = makeQueryClient();
  });

  // -------------------------------------------------------------------------
  // Query: search caches by filters + page + pageSize + keyword
  // -------------------------------------------------------------------------

  it("useTransactionSearch caches by filter+page+pageSize+keyword tuple", async () => {
    adapterMocks.searchTransactions.mockResolvedValue({ items: [sampleTransaction], total: 1 });

    const { result } = renderHook(
      () => useTransactionSearch({ accountIds: ["a1"] }, 0, 50, "coffee"),
      { wrapper: makeWrapper(qc) },
    );

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(adapterMocks.searchTransactions).toHaveBeenCalledWith(
      0,
      50,
      { accountIds: ["a1"] },
      "coffee",
      undefined,
    );
    expect(result.current.data?.total).toBe(1);
  });

  // -------------------------------------------------------------------------
  // Mutation: useCreateTransaction invalidates lists + running balance + accounts
  // -------------------------------------------------------------------------

  it("useCreateTransaction invalidates TRANSACTIONS, running-balance, recent, and ACCOUNTS", async () => {
    adapterMocks.createTransaction.mockResolvedValue(sampleTransaction);
    const invalidateSpy = vi.spyOn(qc, "invalidateQueries");

    const { result } = renderHook(() => useCreateTransaction(), { wrapper: makeWrapper(qc) });
    await result.current.mutateAsync({
      accountId: "a1",
      direction: "EXPENSE",
      amount: 10,
      currency: "USD",
      transactionDate: "2026-04-30",
      source: "MANUAL",
    });

    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["transactions"] });
    expect(invalidateSpy).toHaveBeenCalledWith({
      queryKey: ["transactions", "running-balance"],
    });
    expect(invalidateSpy).toHaveBeenCalledWith({
      queryKey: ["transactions", "by-account-recent"],
    });
    // ACCOUNTS key — current_balance derives from transactions (Phase 3 D-14)
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["accounts"] });
  });

  // -------------------------------------------------------------------------
  // Mutation: useUpdateTransaction also invalidates the specific transaction item
  // -------------------------------------------------------------------------

  it("useUpdateTransaction invalidates lists AND the specific transaction item", async () => {
    adapterMocks.updateTransaction.mockResolvedValue(sampleTransaction);
    const invalidateSpy = vi.spyOn(qc, "invalidateQueries");

    const { result } = renderHook(() => useUpdateTransaction(), { wrapper: makeWrapper(qc) });
    await result.current.mutateAsync({
      transaction: { id: "t1", amount: 20 },
    });

    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["transactions"] });
    expect(invalidateSpy).toHaveBeenCalledWith({
      queryKey: ["transactions", "running-balance"],
    });
    expect(invalidateSpy).toHaveBeenCalledWith({
      queryKey: ["transactions", "by-account-recent"],
    });
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["accounts"] });
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["transactions", "item", "t1"] });
  });

  // -------------------------------------------------------------------------
  // Mutation: useDeleteTransaction invalidates lists + running balance
  // -------------------------------------------------------------------------

  it("useDeleteTransaction invalidates TRANSACTIONS + running-balance + recent + ACCOUNTS", async () => {
    adapterMocks.deleteTransaction.mockResolvedValue(sampleTransaction);
    const invalidateSpy = vi.spyOn(qc, "invalidateQueries");

    const { result } = renderHook(() => useDeleteTransaction(), { wrapper: makeWrapper(qc) });
    await result.current.mutateAsync("t1");

    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["transactions"] });
    expect(invalidateSpy).toHaveBeenCalledWith({
      queryKey: ["transactions", "running-balance"],
    });
    expect(invalidateSpy).toHaveBeenCalledWith({
      queryKey: ["transactions", "by-account-recent"],
    });
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["accounts"] });
  });

  // -------------------------------------------------------------------------
  // Mutation: useCreateTransfer invalidates running-balance (covers both legs
  // because the prefix-matching invalidation hits all running-balance entries)
  // -------------------------------------------------------------------------

  it("useCreateTransfer invalidates running-balance prefix (covers both legs)", async () => {
    adapterMocks.createTransfer.mockResolvedValue([sampleTransaction, sampleTransaction]);
    const invalidateSpy = vi.spyOn(qc, "invalidateQueries");

    const { result } = renderHook(() => useCreateTransfer(), { wrapper: makeWrapper(qc) });
    await result.current.mutateAsync({
      src: { accountId: "a1", amount: 100, currency: "USD", transactionDate: "2026-04-30" },
      dst: { accountId: "a2", amount: 100, currency: "USD", transactionDate: "2026-04-30" },
    });

    // The prefix ["transactions", "running-balance"] matches running-balance for
    // BOTH accounts a1 and a2 (TanStack Query prefix matching).
    expect(invalidateSpy).toHaveBeenCalledWith({
      queryKey: ["transactions", "running-balance"],
    });
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["transactions"] });
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["accounts"] });
  });

  // -------------------------------------------------------------------------
  // Mutation: useImportTransactionsCsv invalidates lists
  // -------------------------------------------------------------------------

  it("useImportTransactionsCsv invalidates transaction lists on success", async () => {
    const result_: TransactionImportResult = {
      importRunId: "ir-1",
      inserted: 3,
      skippedDuplicates: 1,
      errors: [],
      pendingDuplicateCount: 0,
      insertedRowIds: ["t1", "t2", "", "t3"],
    };
    adapterMocks.importTransactionsCsv.mockResolvedValue(result_);
    const invalidateSpy = vi.spyOn(qc, "invalidateQueries");

    const { result } = renderHook(() => useImportTransactionsCsv(), { wrapper: makeWrapper(qc) });
    const returned = await result.current.mutateAsync({
      accountId: "a1",
      file: new Blob(["a,b\n1,2"]),
      mapping: {
        dateColumn: "Date",
        payeeColumn: "Payee",
        dateFormat: "auto",
        decimalSeparator: ".",
      },
    });

    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["transactions"] });
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["accounts"] });
    // insertedRowIds is order-preserving — required by plan 04-09 to wire pending duplicate pairs
    expect(returned.insertedRowIds).toEqual(["t1", "t2", "", "t3"]);
  });
});

// ---------------------------------------------------------------------------
// use-merchant-categories: lookup is gated by enabled + non-empty payee
// ---------------------------------------------------------------------------

describe("use-merchant-categories.useLookupPayeeCategory", () => {
  let qc: QueryClient;

  beforeEach(() => {
    vi.clearAllMocks();
    qc = makeQueryClient();
  });

  it("does NOT fire when payee is empty (D-15: only after user types)", () => {
    renderHook(() => useLookupPayeeCategory("a1", ""), { wrapper: makeWrapper(qc) });
    expect(adapterMocks.lookupPayeeCategory).not.toHaveBeenCalled();
  });

  it("does NOT fire when accountId is null", () => {
    renderHook(() => useLookupPayeeCategory(null, "Starbucks"), { wrapper: makeWrapper(qc) });
    expect(adapterMocks.lookupPayeeCategory).not.toHaveBeenCalled();
  });

  it("fires when accountId AND non-empty payee are provided", async () => {
    adapterMocks.lookupPayeeCategory.mockResolvedValue({
      accountId: "a1",
      normalizedMerchant: "starbucks",
      categoryId: "cat-1",
      lastSeenAt: "2026-04-30T00:00:00Z",
      seenCount: 3,
    });

    const { result } = renderHook(() => useLookupPayeeCategory("a1", "Starbucks"), {
      wrapper: makeWrapper(qc),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(adapterMocks.lookupPayeeCategory).toHaveBeenCalledWith("a1", "Starbucks");
    expect(result.current.data?.categoryId).toBe("cat-1");
  });

  it("respects the explicit enabled=false override", () => {
    renderHook(() => useLookupPayeeCategory("a1", "Starbucks", false), {
      wrapper: makeWrapper(qc),
    });
    expect(adapterMocks.lookupPayeeCategory).not.toHaveBeenCalled();
  });
});
