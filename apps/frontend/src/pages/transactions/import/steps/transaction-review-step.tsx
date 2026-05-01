// Transaction Review Step (Phase 4, plan 04-08).
//
// On mount: calls useDetectTransactionDuplicates with the drafted rows.
// Renders a preview table with duplicate confidence buckets per UI-SPEC §6:
//   ≥95 → bg-destructive/10   (ALMOST_CERTAIN)
//   70-94 → bg-warning/10     (LIKELY)
//   50-69 → bg-muted/50       (POSSIBLE)
//   <50 → not rendered
// Duplicate action buttons: "Discard new" / "Keep both" (D-19 copy verbatim).

import { useEffect } from "react";
import { Button } from "@whaleit/ui/components/ui/button";
import { useDetectTransactionDuplicates } from "@/hooks/use-transactions";
import { useTransactionImport } from "../context/transaction-import-context";
import type { DraftTransaction } from "../context/transaction-import-context";
import type { NewTransaction } from "@/lib/types/transaction";
import { applyDuplicateMatches } from "../utils/transaction-draft-utils";

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

function draftsToNewTransactions(drafts: DraftTransaction[]): NewTransaction[] {
  return drafts
    .filter((d) => !d.skip)
    .map((d) => ({
      accountId: d.accountId,
      direction: d.direction,
      amount: d.amount,
      currency: d.currency,
      transactionDate: d.transactionDate,
      payee: d.payee,
      notes: d.notes,
      categoryId: d.categoryId,
      externalRef: d.externalRef,
      source: "CSV" as const,
    }));
}

function bucketRowClass(confidence: number | null): string {
  if (confidence === null) return "";
  if (confidence >= 95) return "bg-destructive/10";
  if (confidence >= 70) return "bg-warning/10";
  if (confidence >= 50) return "bg-muted/50";
  return "";
}

// ─────────────────────────────────────────────────────────────────────────────
// Component
// ─────────────────────────────────────────────────────────────────────────────

export function TransactionReviewStep() {
  const { state, dispatch } = useTransactionImport();
  const detectDuplicates = useDetectTransactionDuplicates();

  // Run duplicate detection on mount if not yet checked
  useEffect(() => {
    if (state.duplicatesChecked || state.drafts.length === 0) return;
    const candidates = draftsToNewTransactions(state.drafts);
    detectDuplicates.mutate(candidates, {
      onSuccess: (matches) => {
        const updated = applyDuplicateMatches(state.drafts, matches);
        dispatch({ type: "SET_DRAFTS", payload: updated });
        dispatch({ type: "SET_PENDING_DUPLICATES", payload: matches });
        dispatch({ type: "SET_DUPLICATES_CHECKED", payload: true });
      },
    });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const visibleDrafts = state.drafts.filter((d) => {
    // Hide rows with <50 confidence that haven't been acted on
    if (d.duplicateConfidence !== null && d.duplicateConfidence < 50 && d.userResolution === null) {
      return false;
    }
    return true;
  });

  const invalidCount = state.drafts.filter((d) => !d.isValid && !d.skip).length;
  const canContinue = invalidCount === 0;

  function handleSetResolution(rowIndex: number, resolution: "DISCARD_NEW" | "KEEP_BOTH") {
    dispatch({
      type: "UPDATE_DRAFT",
      payload: { rowIndex, updates: { userResolution: resolution } },
    });
  }

  function handleToggleSkip(rowIndex: number, skip: boolean) {
    dispatch({
      type: "UPDATE_DRAFT",
      payload: { rowIndex, updates: { skip } },
    });
  }

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-lg font-semibold">Review transactions</h2>
        <p className="text-muted-foreground mt-1 text-sm">
          Check the rows below before importing. Possible duplicates are highlighted.
        </p>
      </div>

      {detectDuplicates.isPending && (
        <p className="text-muted-foreground text-sm">Checking for duplicates…</p>
      )}

      {/* Preview table */}
      <div className="overflow-x-auto rounded-lg border">
        <table className="w-full text-sm">
          <thead className="bg-muted">
            <tr>
              <th className="px-3 py-2 text-left font-medium">Date</th>
              <th className="px-3 py-2 text-left font-medium">Payee</th>
              <th className="px-3 py-2 text-right font-medium">Amount</th>
              <th className="px-3 py-2 text-left font-medium">Currency</th>
              <th className="px-3 py-2 text-left font-medium">Status</th>
              <th className="px-3 py-2 text-left font-medium">Actions</th>
            </tr>
          </thead>
          <tbody>
            {visibleDrafts.map((draft) => {
              const rowClass = bucketRowClass(draft.duplicateConfidence);
              return (
                <tr
                  key={draft.rowIndex}
                  className={`border-t ${rowClass} ${draft.skip ? "opacity-40" : ""}`}
                >
                  <td className="px-3 py-2 font-mono text-xs">{draft.transactionDate || "—"}</td>
                  <td className="px-3 py-2">{draft.payee ?? "—"}</td>
                  <td className="px-3 py-2 text-right font-mono">
                    {draft.direction === "EXPENSE" ? "-" : "+"}
                    {draft.amount.toFixed(2)}
                  </td>
                  <td className="px-3 py-2">{draft.currency}</td>
                  <td className="px-3 py-2">
                    {draft.duplicateBucket && (
                      <span className="text-xs font-medium">
                        {draft.duplicateBucket === "ALMOST_CERTAIN"
                          ? "Likely duplicate"
                          : draft.duplicateBucket === "LIKELY"
                            ? "Possible duplicate"
                            : "Might be duplicate"}{" "}
                        ({draft.duplicateConfidence}%)
                      </span>
                    )}
                    {!draft.isValid && (
                      <span className="text-destructive text-xs">
                        {draft.validationErrors.join(", ")}
                      </span>
                    )}
                  </td>
                  <td className="px-3 py-2">
                    <div className="flex items-center gap-2">
                      {draft.duplicateBucket && draft.userResolution === null && (
                        <>
                          <Button
                            variant="ghost"
                            size="sm"
                            className="h-6 text-xs"
                            onClick={() => handleSetResolution(draft.rowIndex, "DISCARD_NEW")}
                          >
                            Discard new
                          </Button>
                          <Button
                            variant="ghost"
                            size="sm"
                            className="h-6 text-xs"
                            onClick={() => handleSetResolution(draft.rowIndex, "KEEP_BOTH")}
                          >
                            Keep both
                          </Button>
                        </>
                      )}
                      {draft.userResolution && (
                        <span className="text-muted-foreground text-xs">
                          {draft.userResolution === "DISCARD_NEW" ? "Will skip" : "Will import"}
                        </span>
                      )}
                      <button
                        type="button"
                        className="text-muted-foreground text-xs underline-offset-2 hover:underline"
                        onClick={() => handleToggleSkip(draft.rowIndex, !draft.skip)}
                      >
                        {draft.skip ? "Include" : "Skip"}
                      </button>
                    </div>
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
        {visibleDrafts.length === 0 && (
          <p className="text-muted-foreground py-8 text-center text-sm">
            No transactions to review.
          </p>
        )}
      </div>

      {/* Footer */}
      <div className="flex justify-between pt-4">
        <Button variant="outline" onClick={() => dispatch({ type: "PREV_STEP" })}>
          Back
        </Button>
        <Button onClick={() => dispatch({ type: "NEXT_STEP" })} disabled={!canContinue}>
          Continue to import
        </Button>
      </div>
    </div>
  );
}
