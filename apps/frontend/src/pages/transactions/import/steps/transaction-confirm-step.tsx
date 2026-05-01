// Transaction Confirm Step (Phase 4, plan 04-08).
//
// Summary card + final import CTA.
// Calls useImportTransactionsCsv or useImportTransactionsOfx per state.format.
// On success: navigate to /transactions with toast "{N} transactions imported".
// Stores pendingDuplicateCount in localStorage for plan 04-09 banner.

import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { Button } from "@whaleit/ui/components/ui/button";
import { useImportTransactionsCsv, useImportTransactionsOfx } from "@/hooks/use-transactions";
import { useTransactionImport } from "../context/transaction-import-context";

// ─────────────────────────────────────────────────────────────────────────────
// Component
// ─────────────────────────────────────────────────────────────────────────────

export function TransactionConfirmStep() {
  const { state, dispatch } = useTransactionImport();
  const navigate = useNavigate();
  const importCsv = useImportTransactionsCsv();
  const importOfx = useImportTransactionsOfx();

  const [cancelOpen, setCancelOpen] = useState(false);

  const skippedCount = state.drafts.filter(
    (d) => d.skip || d.userResolution === "DISCARD_NEW",
  ).length;
  const duplicateFlagCount = state.drafts.filter((d) => d.duplicateBucket !== null).length;
  const toImportCount = state.drafts.length - skippedCount;

  const isPending = importCsv.isPending || importOfx.isPending;

  function handleImport() {
    if (!state.file || !state.accountId) return;

    function onSuccess(inserted: number, pendingDuplicateCount: number) {
      // Store pending duplicate count for plan 04-09's banner to pick up
      if (pendingDuplicateCount > 0) {
        localStorage.setItem(
          "whaleit:pendingDuplicates",
          JSON.stringify({ count: pendingDuplicateCount, accountId: state.accountId }),
        );
      }
      void navigate("/transactions");
      // Toast is handled by the app's global toast system via query invalidation
      // We store the result text in sessionStorage for the page to display
      sessionStorage.setItem("whaleit:importSuccess", `${inserted} transactions imported`);
    }

    if (state.format === "OFX") {
      importOfx.mutate(
        { accountId: state.accountId, file: state.file },
        {
          onSuccess: (result) => onSuccess(result.inserted, result.pendingDuplicateCount),
        },
      );
    } else {
      if (!state.mapping) return;
      importCsv.mutate(
        { accountId: state.accountId, file: state.file, mapping: state.mapping },
        {
          onSuccess: (result) => onSuccess(result.inserted, result.pendingDuplicateCount),
        },
      );
    }
  }

  function handleCancel() {
    dispatch({ type: "RESET" });
    void navigate(-1);
  }

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-lg font-semibold">Ready to import</h2>
        <p className="text-muted-foreground mt-1 text-sm">
          Review the summary below and confirm your import.
        </p>
      </div>

      {/* Summary card */}
      <div className="space-y-2 rounded-lg border p-5">
        <div className="flex justify-between text-sm">
          <span className="text-muted-foreground">Transactions to import</span>
          <span className="font-semibold">{toImportCount}</span>
        </div>
        {duplicateFlagCount > 0 && (
          <div className="flex justify-between text-sm">
            <span className="text-muted-foreground">Flagged as possible duplicates</span>
            <span className="font-semibold">{duplicateFlagCount}</span>
          </div>
        )}
        {skippedCount > 0 && (
          <div className="flex justify-between text-sm">
            <span className="text-muted-foreground">Marked to skip</span>
            <span className="font-semibold">{skippedCount}</span>
          </div>
        )}
        <div className="flex justify-between text-sm">
          <span className="text-muted-foreground">Format</span>
          <span className="font-semibold">{state.format}</span>
        </div>
      </div>

      {importCsv.isError && (
        <p className="text-destructive text-sm">
          {importCsv.error?.message ?? "Import failed. Please try again."}
        </p>
      )}
      {importOfx.isError && (
        <p className="text-destructive text-sm">
          {importOfx.error?.message ?? "Import failed. Please try again."}
        </p>
      )}

      {/* Footer */}
      <div className="flex justify-between pt-4">
        <Button variant="outline" onClick={() => setCancelOpen(true)}>
          Cancel
        </Button>
        <Button onClick={handleImport} disabled={isPending || toImportCount === 0}>
          {isPending
            ? "Importing…"
            : `Import ${toImportCount} transaction${toImportCount === 1 ? "" : "s"}`}
        </Button>
      </div>

      {/* Cancel confirmation inline dialog (simplified — no extra dep) */}
      {cancelOpen && (
        <div className="bg-background/80 fixed inset-0 z-50 flex items-center justify-center backdrop-blur-sm">
          <div className="bg-background w-full max-w-sm space-y-4 rounded-xl border p-6 shadow-lg">
            <h3 className="text-base font-semibold">Discard this import?</h3>
            <p className="text-muted-foreground text-sm">
              Your file and column choices will be cleared. You can start over anytime.
            </p>
            <div className="flex justify-end gap-2">
              <Button variant="outline" size="sm" onClick={() => setCancelOpen(false)}>
                Keep editing
              </Button>
              <Button variant="destructive" size="sm" onClick={handleCancel}>
                Discard
              </Button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
