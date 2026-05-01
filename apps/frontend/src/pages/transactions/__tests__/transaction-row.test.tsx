import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { TransactionRow } from "../transaction-row";
import type { Transaction } from "@/lib/types/transaction";

// PrivacyAmount renders amount formatted — mock minimally
vi.mock("@whaleit/ui", () => ({
  PrivacyAmount: ({ value, currency }: { value: number; currency: string }) => (
    <span data-testid="privacy-amount">
      {value} {currency}
    </span>
  ),
}));

function makeTransaction(overrides: Partial<Transaction> = {}): Transaction {
  return {
    id: "txn-1",
    accountId: "acc-1",
    direction: "EXPENSE",
    amount: 50,
    currency: "USD",
    transactionDate: "2024-01-15",
    payee: "Coffee Shop",
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
    createdAt: "2024-01-15T10:00:00Z",
    updatedAt: "2024-01-15T10:00:00Z",
    splits: [],
    ...overrides,
  };
}

describe("TransactionRow", () => {
  it("renders income with ArrowDownLeft and text-success", () => {
    const txn = makeTransaction({ direction: "INCOME", payee: "Salary" });
    const { container } = render(<TransactionRow transaction={txn} baseCurrency="USD" />);
    // ArrowDownLeft is rendered as SVG with specific path data
    // Direction icon is the first element after button
    const row = screen.getByTestId("transaction-row");
    expect(row).toBeTruthy();
    // The icon wrapper has the text-success class
    const icons = container.querySelectorAll("svg");
    const directionIcon = icons[0];
    expect(directionIcon?.className?.baseVal ?? directionIcon?.getAttribute("class")).toContain(
      "text-success",
    );
  });

  it("renders expense with ArrowUpRight and text-muted-foreground on icon", () => {
    const txn = makeTransaction({ direction: "EXPENSE" });
    const { container } = render(<TransactionRow transaction={txn} baseCurrency="USD" />);
    const icons = container.querySelectorAll("svg");
    const directionIcon = icons[0];
    expect(directionIcon?.className?.baseVal ?? directionIcon?.getAttribute("class")).toContain(
      "text-muted-foreground",
    );
  });

  it("renders transfer with ArrowLeftRight and muted amount", () => {
    const txn = makeTransaction({
      direction: "TRANSFER",
      payee: null,
      transferGroupId: "grp-1",
    });
    const { container } = render(<TransactionRow transaction={txn} baseCurrency="USD" />);
    const icons = container.querySelectorAll("svg");
    const directionIcon = icons[0];
    expect(directionIcon?.className?.baseVal ?? directionIcon?.getAttribute("class")).toContain(
      "text-muted-foreground",
    );
    // Amount text should have text-muted-foreground
    const amountWrapper = container.querySelector(".tabular-nums");
    expect(amountWrapper?.className).toContain("text-muted-foreground");
  });

  it("renders fx sub-line when currency != baseCurrency", () => {
    const txn = makeTransaction({ currency: "EUR", fxRate: 1.08 });
    render(<TransactionRow transaction={txn} baseCurrency="USD" />);
    // Should show two PrivacyAmount elements: native + converted
    const amounts = screen.getAllByTestId("privacy-amount");
    expect(amounts.length).toBeGreaterThanOrEqual(2);
    // Second amount should be in USD (base)
    expect(amounts[1].textContent).toContain("USD");
  });

  it("hides running-balance when showRunningBalance=false", () => {
    const txn = makeTransaction();
    render(
      <TransactionRow
        transaction={txn}
        baseCurrency="USD"
        showRunningBalance={false}
        runningBalance={1000}
      />,
    );
    expect(screen.queryByText(/Bal/)).toBeNull();
  });

  it("shows running-balance caption when showRunningBalance=true", () => {
    const txn = makeTransaction();
    render(
      <TransactionRow
        transaction={txn}
        baseCurrency="USD"
        showRunningBalance={true}
        runningBalance={1000}
      />,
    );
    expect(screen.getByText(/Bal/)).toBeTruthy();
  });

  it("renders split badge instead of category chip when hasSplits=true", () => {
    const txn = makeTransaction({
      hasSplits: true,
      splits: [
        {
          id: "s1",
          transactionId: "txn-1",
          categoryId: "c1",
          amount: 25,
          notes: null,
          sortOrder: 0,
        },
        {
          id: "s2",
          transactionId: "txn-1",
          categoryId: "c2",
          amount: 25,
          notes: null,
          sortOrder: 1,
        },
      ],
    });
    render(<TransactionRow transaction={txn} baseCurrency="USD" categoryName="Food" />);
    expect(screen.getByText(/Split · 2 categories/)).toBeTruthy();
    expect(screen.queryByText("Food")).toBeNull();
  });

  it("renders notes glyph only when notes is non-empty", () => {
    const withNotes = makeTransaction({ notes: "Receipt scanned" });
    const withoutNotes = makeTransaction({ notes: null });

    const { rerender, container } = render(
      <TransactionRow transaction={withNotes} baseCurrency="USD" />,
    );
    // MessageSquare icon has aria-label "has note"
    expect(screen.getByLabelText("has note")).toBeTruthy();

    rerender(<TransactionRow transaction={withoutNotes} baseCurrency="USD" />);
    expect(screen.queryByLabelText("has note")).toBeNull();
    void container; // suppress unused var
  });

  it("account-suppressed variant hides account name", () => {
    const txn = makeTransaction();
    render(
      <TransactionRow
        transaction={txn}
        baseCurrency="USD"
        variant="account-suppressed"
        accountName="My Checking"
      />,
    );
    expect(screen.queryByText("My Checking")).toBeNull();
  });

  it("default variant shows account name when provided", () => {
    const txn = makeTransaction();
    render(
      <TransactionRow
        transaction={txn}
        baseCurrency="USD"
        variant="default"
        accountName="My Checking"
      />,
    );
    expect(screen.getByText("My Checking")).toBeTruthy();
  });

  it("D-03 transfer-pair indicator renders only when transferGroupId is non-null AND direction is TRANSFER", () => {
    // TRANSFER with transferGroupId → shows ↔
    const withPair = makeTransaction({
      direction: "TRANSFER",
      transferGroupId: "grp-1",
    });
    const { rerender } = render(<TransactionRow transaction={withPair} baseCurrency="USD" />);
    expect(screen.getByLabelText("transfer pair")).toBeTruthy();

    // TRANSFER without transferGroupId → no glyph
    const withoutPair = makeTransaction({
      direction: "TRANSFER",
      transferGroupId: null,
    });
    rerender(<TransactionRow transaction={withoutPair} baseCurrency="USD" />);
    expect(screen.queryByLabelText("transfer pair")).toBeNull();

    // Non-TRANSFER with transferGroupId → no glyph
    const expenseWithGroup = makeTransaction({
      direction: "EXPENSE",
      transferGroupId: "grp-2",
    });
    rerender(<TransactionRow transaction={expenseWithGroup} baseCurrency="USD" />);
    expect(screen.queryByLabelText("transfer pair")).toBeNull();
  });

  it("duplicate tinting applies correct bg class based on confidence", () => {
    const txn = makeTransaction();

    const { container: c95 } = render(
      <TransactionRow transaction={txn} baseCurrency="USD" duplicateConfidence={95} />,
    );
    expect(c95.querySelector("[data-testid='transaction-row']")?.className).toContain(
      "bg-destructive/10",
    );

    const { container: c80 } = render(
      <TransactionRow transaction={txn} baseCurrency="USD" duplicateConfidence={80} />,
    );
    expect(c80.querySelector("[data-testid='transaction-row']")?.className).toContain(
      "bg-warning/10",
    );

    const { container: c60 } = render(
      <TransactionRow transaction={txn} baseCurrency="USD" duplicateConfidence={60} />,
    );
    expect(c60.querySelector("[data-testid='transaction-row']")?.className).toContain(
      "bg-muted/50",
    );

    const { container: c30 } = render(
      <TransactionRow transaction={txn} baseCurrency="USD" duplicateConfidence={30} />,
    );
    const btn30 = c30.querySelector("[data-testid='transaction-row']");
    expect(btn30?.className).not.toContain("bg-destructive");
    expect(btn30?.className).not.toContain("bg-warning");
    expect(btn30?.className).not.toContain("bg-muted/50");
  });
});
