// Transaction Mapping Step (Phase 4, plan 04-08).
//
// D-17: shows inline mismatch banner when a saved template's header signature
// doesn't match the file's current column positions.
// Continue button disabled until date + payee + (amount OR debit+credit) are mapped.

import { useEffect, useState } from "react";
import { Button } from "@whaleit/ui/components/ui/button";
import { TransactionMappingTable } from "../components/transaction-mapping-table";
import { TransactionTemplatePicker } from "../components/transaction-template-picker";
import { useTransactionImport } from "../context/transaction-import-context";
import { useTransactionImportMapping } from "../hooks/use-transaction-import-mapping";
import { createDefaultTransactionMapping } from "../utils/transaction-default-template";
import type { CsvFieldMapping } from "@/lib/types/transaction";

// ─────────────────────────────────────────────────────────────────────────────
// Required-field validation
// ─────────────────────────────────────────────────────────────────────────────

function isMappingComplete(mapping: CsvFieldMapping | null): boolean {
  if (!mapping) return false;
  const hasDate = !!mapping.dateColumn;
  const hasPayee = !!mapping.payeeColumn;
  const hasAmount = !!mapping.amountColumn || !!(mapping.debitColumn || mapping.creditColumn);
  return hasDate && hasPayee && hasAmount;
}

// ─────────────────────────────────────────────────────────────────────────────
// Component
// ─────────────────────────────────────────────────────────────────────────────

export function TransactionMappingStep() {
  const { state, dispatch } = useTransactionImport();
  const { setMapping } = useTransactionImportMapping();
  const [useSplitAmount, setUseSplitAmount] = useState(false);

  // Initialize mapping with defaults if not yet set
  useEffect(() => {
    if (!state.mapping) {
      dispatch({ type: "SET_MAPPING", payload: createDefaultTransactionMapping() });
    }
  }, [state.mapping, dispatch]);

  const activeMapping: CsvFieldMapping = state.mapping ?? createDefaultTransactionMapping();
  const canContinue = isMappingComplete(state.mapping);

  function handleContinue() {
    if (!canContinue) return;
    dispatch({ type: "NEXT_STEP" });
  }

  function handleClearTemplate() {
    dispatch({ type: "CLEAR_TEMPLATE" });
  }

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-lg font-semibold">Map your columns</h2>
        <p className="text-muted-foreground mt-1 text-sm">
          Tell us which CSV columns correspond to each transaction field.
        </p>
      </div>

      {/* D-17: header-signature mismatch banner */}
      {state.headerSignatureMismatch && state.selectedTemplateName && (
        <div className="bg-warning/10 flex items-start gap-3 rounded-lg border px-4 py-3">
          <p className="flex-1 text-sm" data-testid="header-mismatch-message">
            {`Your saved '${state.selectedTemplateName}' template doesn't match this file's columns. Re-map?`}
          </p>
          <Button variant="ghost" size="sm" onClick={handleClearTemplate} className="shrink-0">
            Re-map
          </Button>
        </div>
      )}

      {/* Template picker */}
      <TransactionTemplatePicker currentHeaders={state.csvHeaders} currentMapping={state.mapping} />

      {/* Mapping table — always render with activeMapping (defaults to empty if not yet set) */}
      <TransactionMappingTable
        headers={state.csvHeaders}
        mapping={activeMapping}
        onMappingChange={(updated) => setMapping(updated)}
        useSplitAmountColumns={useSplitAmount}
        onToggleSplitAmount={setUseSplitAmount}
      />

      {/* Footer navigation */}
      <div className="flex justify-between pt-4">
        <Button variant="outline" onClick={() => dispatch({ type: "PREV_STEP" })}>
          Back
        </Button>
        <Button onClick={handleContinue} disabled={!canContinue}>
          Continue to review
        </Button>
      </div>
    </div>
  );
}
