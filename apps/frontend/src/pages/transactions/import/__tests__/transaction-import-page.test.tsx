// Transaction Import Page tests (Phase 4, plan 04-08, Task 4).
//
// D-19: OFX format SKIPS Mapping step. Step indicator must NOT render "Mapping".
// CSV format DOES render Mapping. Both cases asserted in this file.

import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MemoryRouter } from "react-router-dom";

// Mock hooks the wizard's child steps depend on (not the focus of this test)
vi.mock("@/hooks/use-transaction-templates", () => ({
  useTransactionTemplates: () => ({ data: [], isLoading: false }),
  useSaveTransactionTemplate: () => ({ mutate: vi.fn(), isPending: false }),
  useDeleteTransactionTemplate: () => ({ mutate: vi.fn(), isPending: false }),
}));

vi.mock("@/hooks/use-transactions", () => ({
  useDetectTransactionDuplicates: () => ({ mutate: vi.fn(), isPending: false }),
  useImportTransactionsCsv: () => ({
    mutate: vi.fn(),
    isPending: false,
    isError: false,
    error: null,
  }),
  useImportTransactionsOfx: () => ({
    mutate: vi.fn(),
    isPending: false,
    isError: false,
    error: null,
  }),
}));

vi.mock("@/adapters", () => ({
  logger: { error: vi.fn(), warn: vi.fn(), info: vi.fn(), debug: vi.fn(), trace: vi.fn() },
}));

function makeQc() {
  return new QueryClient({ defaultOptions: { queries: { retry: false } } });
}

async function renderWithFormat(format: "CSV" | "OFX") {
  // Lazy-import after mocks are set up
  const { TransactionImportProvider } = await import("../context/transaction-import-context");
  const { WizardContent } = await import("../transaction-import-page");

  const qc = makeQc();
  return render(
    <QueryClientProvider client={qc}>
      <MemoryRouter>
        <TransactionImportProvider initialState={{ format }}>
          <WizardContent />
        </TransactionImportProvider>
      </MemoryRouter>
    </QueryClientProvider>,
  );
}

describe("TransactionImportPage step indicator", () => {
  it("D-19: OFX format wizard does NOT render the Mapping step", async () => {
    await renderWithFormat("OFX");
    // Step indicator must not contain "Mapping"
    expect(screen.queryByText("Mapping")).toBeNull();
    // Upload, Review, Confirm should all be present
    expect(screen.getByText("Upload")).toBeInTheDocument();
    expect(screen.getByText(/Review transactions/i)).toBeInTheDocument();
    expect(screen.getByText("Import")).toBeInTheDocument();
  });

  it("CSV format wizard DOES render all four steps including Mapping", async () => {
    await renderWithFormat("CSV");
    expect(screen.getByText("Upload")).toBeInTheDocument();
    expect(screen.getByText("Mapping")).toBeInTheDocument();
    expect(screen.getByText(/Review transactions/i)).toBeInTheDocument();
    expect(screen.getByText("Import")).toBeInTheDocument();
  });
});
