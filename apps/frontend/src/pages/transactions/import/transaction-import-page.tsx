// Transaction Import Page (Phase 4, plan 04-08).
//
// Wizard root — drives step orchestration.
// CSV path: Upload -> Mapping -> Review -> Confirm
// OFX path: Upload -> Review -> Confirm  (D-19: OFX skips Mapping)
//
// Wraps everything in TransactionImportProvider.
// ?accountId=X query param pre-scopes the import to an account.

import { useSearchParams } from "react-router-dom";
import {
  TransactionImportProvider,
  useTransactionImport,
} from "./context/transaction-import-context";
import { UploadStep } from "./steps/upload-step";
import { TransactionMappingStep } from "./steps/transaction-mapping-step";
import { TransactionReviewStep } from "./steps/transaction-review-step";
import { TransactionConfirmStep } from "./steps/transaction-confirm-step";

// ----------------------------------------------------------------------------
// Step configuration
// ----------------------------------------------------------------------------

export const CSV_STEPS = [
  { id: "upload", label: "Upload" },
  { id: "mapping", label: "Mapping" },
  { id: "review", label: "Review transactions" },
  { id: "confirm", label: "Import" },
] as const;

export const OFX_STEPS = [
  { id: "upload", label: "Upload" },
  { id: "review", label: "Review transactions" },
  { id: "confirm", label: "Import" },
] as const;

const STEP_COMPONENTS = {
  upload: UploadStep,
  mapping: TransactionMappingStep,
  review: TransactionReviewStep,
  confirm: TransactionConfirmStep,
} as const;

// ----------------------------------------------------------------------------
// Wizard content (reads context — must be inside provider)
// ----------------------------------------------------------------------------

function WizardContent() {
  const { state } = useTransactionImport();
  const steps = state.format === "OFX" ? OFX_STEPS : CSV_STEPS;
  const currentStepIndex = steps.findIndex((s) => s.id === state.currentStep);

  const StepComponent =
    STEP_COMPONENTS[state.currentStep as keyof typeof STEP_COMPONENTS] ?? UploadStep;

  return (
    <div className="mx-auto max-w-3xl px-4 py-8">
      {/* Page header */}
      <div className="mb-6">
        <h1 className="text-2xl font-semibold">Import transactions</h1>
        <p className="text-muted-foreground mt-1 text-sm">
          Bring in your CSV or OFX file from your bank.
        </p>
      </div>

      {/* Step indicator */}
      <nav aria-label="Import steps" className="mb-8">
        <ol className="flex items-center gap-0">
          {steps.map((step, i) => {
            const isActive = step.id === state.currentStep;
            const isPast = i < currentStepIndex;
            return (
              <li key={step.id} className="flex items-center">
                <div className="flex items-center gap-2">
                  <span
                    className={`flex h-6 w-6 items-center justify-center rounded-full text-xs font-semibold ${
                      isActive
                        ? "bg-primary text-primary-foreground"
                        : isPast
                          ? "bg-primary/30 text-primary"
                          : "bg-muted text-muted-foreground"
                    }`}
                  >
                    {i + 1}
                  </span>
                  <span
                    className={`text-sm ${isActive ? "font-semibold" : "text-muted-foreground"}`}
                  >
                    {step.label}
                  </span>
                </div>
                {i < steps.length - 1 && (
                  <span className="text-muted-foreground mx-3 text-xs">{"->"}</span>
                )}
              </li>
            );
          })}
        </ol>
      </nav>

      {/* Active step */}
      <StepComponent />
    </div>
  );
}

// ----------------------------------------------------------------------------
// Page root
// ----------------------------------------------------------------------------

export default function TransactionImportPage() {
  const [searchParams] = useSearchParams();
  const accountIdFromQuery = searchParams.get("accountId");

  return (
    <TransactionImportProvider initialAccountId={accountIdFromQuery}>
      <WizardContent />
    </TransactionImportProvider>
  );
}
