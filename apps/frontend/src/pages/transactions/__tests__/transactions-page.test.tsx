import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter } from "react-router-dom";
import type { Transaction } from "@/lib/types/transaction";

// Hoisted mocks for hooks
const hookMocks = vi.hoisted(() => ({
  useTransactionSearch: vi.fn(),
  useAccounts: vi.fn(),
  useTaxonomy: vi.fn(),
  useSettingsContext: vi.fn(),
  useDeleteTransaction: vi.fn(),
}));

vi.mock("@/hooks/use-transactions", () => ({
  useTransactionSearch: hookMocks.useTransactionSearch,
  useDeleteTransaction: hookMocks.useDeleteTransaction,
}));
vi.mock("@/hooks/use-accounts", () => ({
  useAccounts: hookMocks.useAccounts,
}));
vi.mock("@/hooks/use-taxonomies", () => ({
  useTaxonomy: hookMocks.useTaxonomy,
}));
vi.mock("@/lib/settings-provider", () => ({
  useSettingsContext: hookMocks.useSettingsContext,
}));

// The detail sheet uses TanStack Query directly for sibling lookup. Mock it.
vi.mock("@tanstack/react-query", async (importOriginal) => {
  const actual = await importOriginal<Record<string, unknown>>();
  return {
    ...actual,
    useQuery: () => ({ data: null, isLoading: false }),
  };
});

// Adapter — used by transaction-detail-sheet's sibling fetch.
vi.mock("@/adapters", () => ({
  searchTransactions: vi.fn().mockResolvedValue({ items: [], total: 0 }),
}));

// PrivacyAmount mock for cleaner output
vi.mock("@whaleit/ui", async (importOriginal) => {
  const actual = await importOriginal<Record<string, unknown>>();
  return {
    ...actual,
    PrivacyAmount: ({ value, currency }: { value: number; currency: string }) => (
      <span>
        {value} {currency}
      </span>
    ),
  };
});

import TransactionsPage from "../transactions-page";

function makeTxn(overrides: Partial<Transaction> = {}): Transaction {
  return {
    id: "t1",
    accountId: "acc-1",
    direction: "EXPENSE",
    amount: 50,
    currency: "USD",
    transactionDate: "2024-01-15",
    payee: "Coffee Shop",
    notes: null,
    categoryId: null,
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
    createdAt: "2024-01-15T10:00:00Z",
    updatedAt: "2024-01-15T10:00:00Z",
    splits: [],
    ...overrides,
  };
}

function renderPage() {
  return render(
    <MemoryRouter initialEntries={["/transactions"]}>
      <TransactionsPage />
    </MemoryRouter>,
  );
}

describe("TransactionsPage", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    hookMocks.useTransactionSearch.mockReturnValue({
      data: { items: [], total: 0 },
      isLoading: false,
    });
    hookMocks.useAccounts.mockReturnValue({
      accounts: [
        {
          id: "acc-1",
          name: "Checking",
          currency: "USD",
          isActive: true,
        },
      ],
      isLoading: false,
    });
    hookMocks.useTaxonomy.mockReturnValue({
      data: { taxonomy: { id: "sys_taxonomy_transaction_categories" }, categories: [] },
    });
    hookMocks.useSettingsContext.mockReturnValue({
      settings: { baseCurrency: "USD" },
    });
    hookMocks.useDeleteTransaction.mockReturnValue({
      mutate: vi.fn(),
      mutateAsync: vi.fn().mockResolvedValue(undefined),
      isPending: false,
    });
  });

  it("renders empty state when no transactions", () => {
    renderPage();
    expect(screen.getByText(/No transactions found/)).toBeTruthy();
  });

  it("renders rows when search returns transactions", () => {
    hookMocks.useTransactionSearch.mockReturnValue({
      data: {
        items: [
          makeTxn({ id: "t1", payee: "Coffee Shop" }),
          makeTxn({ id: "t2", payee: "Grocery Store", transactionDate: "2024-01-14" }),
          makeTxn({ id: "t3", payee: "Gas Station", transactionDate: "2024-01-13" }),
        ],
        total: 3,
      },
      isLoading: false,
    });
    renderPage();
    expect(screen.getByText("Coffee Shop")).toBeTruthy();
    expect(screen.getByText("Grocery Store")).toBeTruthy();
    expect(screen.getByText("Gas Station")).toBeTruthy();
  });

  it("renders the FilterBar with account filter chip", () => {
    renderPage();
    expect(screen.getByText("All accounts")).toBeTruthy();
    expect(screen.getByText("Last 30 days")).toBeTruthy();
    expect(screen.getByText("Any category")).toBeTruthy();
  });

  it("does not render duplicate banner when pendingCount is 0", () => {
    renderPage();
    expect(screen.queryByTestId("duplicate-banner")).toBeNull();
  });

  it("renders Import and New transaction buttons in the header", () => {
    renderPage();
    expect(screen.getByRole("button", { name: /Import/ })).toBeTruthy();
    expect(screen.getByRole("button", { name: /New transaction/ })).toBeTruthy();
  });

  it("calls useTransactionSearch with current filters and search keyword", () => {
    renderPage();
    expect(hookMocks.useTransactionSearch).toHaveBeenCalled();
    // First call: filters object, page 0, pageSize 50, empty searchKw
    const lastCall =
      hookMocks.useTransactionSearch.mock.calls[
        hookMocks.useTransactionSearch.mock.calls.length - 1
      ];
    expect(lastCall[1]).toBe(0); // page
    expect(lastCall[2]).toBe(50); // pageSize
    expect(lastCall[3]).toBe(""); // searchKw
  });

  it("typing in search re-invokes useTransactionSearch with new keyword (debounced)", async () => {
    const user = userEvent.setup();
    renderPage();
    const searchInput = screen.getByPlaceholderText("Search transactions...");
    await user.type(searchInput, "Coffee");
    // Debounce is 250ms
    await waitFor(
      () => {
        const calls = hookMocks.useTransactionSearch.mock.calls;
        const found = calls.some((call) => call[3] === "Coffee");
        expect(found).toBe(true);
      },
      { timeout: 1500 },
    );
  });
});

describe("DuplicateBanner integration", () => {
  it("renders duplicate banner copy when pendingCount > 0", async () => {
    // Verify the banner component renders correctly when given a non-zero count.
    // (Wired into TransactionsPage by plan 04-09 — for now the page passes 0.)
    const { DuplicateBanner } = await import("../duplicate-banner");
    render(
      <MemoryRouter>
        <DuplicateBanner pendingCount={3} />
      </MemoryRouter>,
    );
    expect(screen.getByText(/3 possible duplicates/)).toBeTruthy();
    expect(screen.getByTestId("duplicate-banner")).toBeTruthy();
  });

  it("does not render when pendingCount is 0", async () => {
    const { DuplicateBanner } = await import("../duplicate-banner");
    const { container } = render(
      <MemoryRouter>
        <DuplicateBanner pendingCount={0} />
      </MemoryRouter>,
    );
    expect(container.querySelector("[data-testid='duplicate-banner']")).toBeNull();
  });

  it("uses singular 'duplicate' when count is 1", async () => {
    const { DuplicateBanner } = await import("../duplicate-banner");
    render(
      <MemoryRouter>
        <DuplicateBanner pendingCount={1} />
      </MemoryRouter>,
    );
    expect(screen.getByText(/1 possible duplicate from your last import/)).toBeTruthy();
  });
});
