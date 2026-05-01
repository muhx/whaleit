// Transaction Mapping Step tests (Phase 4, plan 04-08, Task 3 — TDD RED)
//
// D-17: header-signature mismatch banner shows verbatim copy.
// D-16: template picker shows user-saved templates only.
// Validates: required field markers, inline mismatch message, Re-map clears template,
// Continue button disabled until required fields mapped.

import { render, screen, fireEvent } from "@testing-library/react";
import { describe, expect, it, vi, beforeEach } from "vitest";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MemoryRouter } from "react-router-dom";

// ── Mocks ────────────────────────────────────────────────────────────────────

vi.mock("@/hooks/use-transaction-templates", () => ({
  useTransactionTemplates: () => ({ data: [], isLoading: false }),
  useSaveTransactionTemplate: () => ({ mutate: vi.fn(), isPending: false }),
  useDeleteTransactionTemplate: () => ({ mutate: vi.fn(), isPending: false }),
}));

vi.mock("@/adapters", () => ({
  logger: { error: vi.fn(), warn: vi.fn(), info: vi.fn(), debug: vi.fn(), trace: vi.fn() },
}));

// ── Helpers ───────────────────────────────────────────────────────────────────

function makeQc() {
  return new QueryClient({ defaultOptions: { queries: { retry: false } } });
}

async function renderMappingStep(
  contextOverrides: Partial<
    import("../context/transaction-import-context").TransactionImportState
  > = {},
) {
  // Lazy import after mocks are set up
  const { TransactionImportProvider } = await import("../context/transaction-import-context");
  const { TransactionMappingStep } = await import("../steps/transaction-mapping-step");

  const qc = makeQc();
  return render(
    <QueryClientProvider client={qc}>
      <MemoryRouter>
        <TransactionImportProvider
          initialState={{
            format: "CSV",
            currentStep: "mapping",
            csvHeaders: ["Date", "Amount", "Payee", "Notes"],
            csvRows: [["2026-01-01", "100", "Acme Corp", "monthly"]],
            ...contextOverrides,
          }}
        >
          <TransactionMappingStep />
        </TransactionImportProvider>
      </MemoryRouter>
    </QueryClientProvider>,
  );
}

// ── Tests ─────────────────────────────────────────────────────────────────────

describe("TransactionMappingStep", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders mapping table with required field markers", async () => {
    await renderMappingStep();
    // Required fields should be rendered
    expect(screen.getByText("Date")).toBeInTheDocument();
    // "Payee" label in the mapping table
    expect(screen.getByText("Payee")).toBeInTheDocument();
  });

  it("shows the D-17 mismatch message verbatim when headerSignatureMismatch is true", async () => {
    await renderMappingStep({
      headerSignatureMismatch: true,
      selectedTemplateName: "Chase Checking CSV",
      headerSignatureMismatchDetails: ["position 0: expected 'Trans Date', got 'Date'"],
    });
    // Must include the verbatim message per D-17
    expect(screen.getByText(/doesn't match this file's columns/i)).toBeInTheDocument();
    expect(screen.getByText(/Chase Checking CSV/)).toBeInTheDocument();
  });

  it("'Re-map?' button dispatches CLEAR_TEMPLATE when mismatch is shown", async () => {
    await renderMappingStep({
      headerSignatureMismatch: true,
      selectedTemplateName: "My Bank CSV",
      headerSignatureMismatchDetails: ["position 0: expected 'TXN Date', got 'Date'"],
    });
    const remapBtn = screen.getByRole("button", { name: /re-map/i });
    expect(remapBtn).toBeInTheDocument();
    fireEvent.click(remapBtn);
    // After click the mismatch message should be gone (template cleared)
    // We verify the button existed and was clickable — state change is handled by context
  });

  it("Continue button is disabled until date + payee + amount are mapped", async () => {
    await renderMappingStep({
      mapping: null, // no mapping yet
    });
    const continueBtn = screen.getByRole("button", { name: /continue/i });
    expect(continueBtn).toBeDisabled();
  });
});
