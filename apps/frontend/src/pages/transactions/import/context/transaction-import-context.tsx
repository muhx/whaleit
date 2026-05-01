// Transaction Import Context (Phase 4, plan 04-08)
//
// Forked from apps/frontend/src/pages/activity/import/context/import-context.tsx.
// Drops all asset/symbol/holdings fields; adds transaction-specific state:
// format (CSV | OFX), duplicate detection results, CsvFieldMapping, template.

import { createContext, useContext, useReducer, type Dispatch, type ReactNode } from "react";
import type { CsvFieldMapping, DuplicateCandidate, DuplicateBucket } from "@/lib/types/transaction";

// ─────────────────────────────────────────────────────────────────────────────
// Types
// ─────────────────────────────────────────────────────────────────────────────

export type ImportFormat = "CSV" | "OFX";
export type WizardStep = "upload" | "mapping" | "review" | "confirm";

export interface DraftTransaction {
  rowIndex: number;
  accountId: string;
  direction: "INCOME" | "EXPENSE" | "TRANSFER";
  amount: number;
  currency: string;
  transactionDate: string;
  payee: string | null;
  notes: string | null;
  categoryId: string | null;
  externalRef: string | null;
  // Validation state
  isValid: boolean;
  validationErrors: string[];
  skip: boolean;
  // Duplicate detection
  duplicateBucket: DuplicateBucket | null;
  duplicateConfidence: number | null;
  existingTransactionId: string | null;
  userResolution: "DISCARD_NEW" | "KEEP_BOTH" | null;
}

export interface TransactionImportState {
  format: ImportFormat;
  currentStep: WizardStep;
  accountId: string | null;
  file: File | null;
  rawCsvText: string | null;
  csvHeaders: string[];
  csvRows: string[][];
  ofxParseResult: { transactions: unknown[] } | null;
  mapping: CsvFieldMapping | null;
  selectedTemplateId: string | null;
  selectedTemplateName: string | null;
  headerSignatureMismatch: boolean;
  headerSignatureMismatchDetails: string[];
  drafts: DraftTransaction[];
  pendingDuplicates: DuplicateCandidate[];
  duplicatesChecked: boolean;
  importRunId: string | null;
}

// ─────────────────────────────────────────────────────────────────────────────
// Actions
// ─────────────────────────────────────────────────────────────────────────────

export type TransactionImportAction =
  | { type: "SET_FORMAT"; payload: ImportFormat }
  | { type: "SET_STEP"; payload: WizardStep }
  | { type: "NEXT_STEP" }
  | { type: "PREV_STEP" }
  | { type: "SET_ACCOUNT_ID"; payload: string }
  | { type: "SET_FILE"; payload: File | null }
  | { type: "SET_PARSED_CSV"; payload: { headers: string[]; rows: string[][]; rawText: string } }
  | { type: "SET_OFX_PARSE_RESULT"; payload: { transactions: unknown[] } }
  | { type: "SET_MAPPING"; payload: CsvFieldMapping }
  | { type: "SET_TEMPLATE"; payload: { id: string; name: string; mapping: CsvFieldMapping } }
  | { type: "CLEAR_TEMPLATE" }
  | { type: "SET_HEADER_SIGNATURE_MISMATCH"; payload: { mismatch: boolean; details: string[] } }
  | { type: "SET_DRAFTS"; payload: DraftTransaction[] }
  | { type: "UPDATE_DRAFT"; payload: { rowIndex: number; updates: Partial<DraftTransaction> } }
  | { type: "SET_PENDING_DUPLICATES"; payload: DuplicateCandidate[] }
  | { type: "SET_DUPLICATES_CHECKED"; payload: boolean }
  | { type: "SET_IMPORT_RUN_ID"; payload: string }
  | { type: "RESET" };

// ─────────────────────────────────────────────────────────────────────────────
// Step navigation helpers
// ─────────────────────────────────────────────────────────────────────────────

const CSV_STEP_ORDER: WizardStep[] = ["upload", "mapping", "review", "confirm"];
const OFX_STEP_ORDER: WizardStep[] = ["upload", "review", "confirm"];

function getNextStep(current: WizardStep, format: ImportFormat): WizardStep {
  const order = format === "OFX" ? OFX_STEP_ORDER : CSV_STEP_ORDER;
  const idx = order.indexOf(current);
  if (idx >= 0 && idx < order.length - 1) {
    return order[idx + 1];
  }
  return current;
}

function getPrevStep(current: WizardStep, format: ImportFormat): WizardStep {
  const order = format === "OFX" ? OFX_STEP_ORDER : CSV_STEP_ORDER;
  const idx = order.indexOf(current);
  if (idx > 0) {
    return order[idx - 1];
  }
  return current;
}

// ─────────────────────────────────────────────────────────────────────────────
// Initial state
// ─────────────────────────────────────────────────────────────────────────────

const INITIAL_STATE: TransactionImportState = {
  format: "CSV",
  currentStep: "upload",
  accountId: null,
  file: null,
  rawCsvText: null,
  csvHeaders: [],
  csvRows: [],
  ofxParseResult: null,
  mapping: null,
  selectedTemplateId: null,
  selectedTemplateName: null,
  headerSignatureMismatch: false,
  headerSignatureMismatchDetails: [],
  drafts: [],
  pendingDuplicates: [],
  duplicatesChecked: false,
  importRunId: null,
};

// ─────────────────────────────────────────────────────────────────────────────
// Reducer
// ─────────────────────────────────────────────────────────────────────────────

function reducer(
  state: TransactionImportState,
  action: TransactionImportAction,
): TransactionImportState {
  switch (action.type) {
    case "SET_FORMAT":
      return { ...state, format: action.payload };

    case "SET_STEP":
      return { ...state, currentStep: action.payload };

    case "NEXT_STEP":
      return { ...state, currentStep: getNextStep(state.currentStep, state.format) };

    case "PREV_STEP":
      return { ...state, currentStep: getPrevStep(state.currentStep, state.format) };

    case "SET_ACCOUNT_ID":
      return { ...state, accountId: action.payload };

    case "SET_FILE":
      return { ...state, file: action.payload };

    case "SET_PARSED_CSV":
      return {
        ...state,
        csvHeaders: action.payload.headers,
        csvRows: action.payload.rows,
        rawCsvText: action.payload.rawText,
        format: "CSV",
      };

    case "SET_OFX_PARSE_RESULT":
      return { ...state, ofxParseResult: action.payload, format: "OFX" };

    case "SET_MAPPING":
      return { ...state, mapping: action.payload };

    case "SET_TEMPLATE":
      return {
        ...state,
        selectedTemplateId: action.payload.id,
        selectedTemplateName: action.payload.name,
        mapping: action.payload.mapping,
        headerSignatureMismatch: false,
        headerSignatureMismatchDetails: [],
      };

    case "CLEAR_TEMPLATE":
      return {
        ...state,
        selectedTemplateId: null,
        selectedTemplateName: null,
        headerSignatureMismatch: false,
        headerSignatureMismatchDetails: [],
      };

    case "SET_HEADER_SIGNATURE_MISMATCH":
      return {
        ...state,
        headerSignatureMismatch: action.payload.mismatch,
        headerSignatureMismatchDetails: action.payload.details,
      };

    case "SET_DRAFTS":
      return { ...state, drafts: action.payload };

    case "UPDATE_DRAFT": {
      const { rowIndex, updates } = action.payload;
      return {
        ...state,
        drafts: state.drafts.map((d) => (d.rowIndex === rowIndex ? { ...d, ...updates } : d)),
      };
    }

    case "SET_PENDING_DUPLICATES":
      return { ...state, pendingDuplicates: action.payload };

    case "SET_DUPLICATES_CHECKED":
      return { ...state, duplicatesChecked: action.payload };

    case "SET_IMPORT_RUN_ID":
      return { ...state, importRunId: action.payload };

    case "RESET":
      return { ...INITIAL_STATE };

    default:
      return state;
  }
}

// ─────────────────────────────────────────────────────────────────────────────
// Context
// ─────────────────────────────────────────────────────────────────────────────

interface TransactionImportContextValue {
  state: TransactionImportState;
  dispatch: Dispatch<TransactionImportAction>;
}

export const TransactionImportContext = createContext<TransactionImportContextValue | null>(null);

// ─────────────────────────────────────────────────────────────────────────────
// Provider
// ─────────────────────────────────────────────────────────────────────────────

interface TransactionImportProviderProps {
  children: ReactNode;
  initialAccountId?: string | null;
  initialState?: Partial<TransactionImportState>;
}

export function TransactionImportProvider({
  children,
  initialAccountId,
  initialState,
}: TransactionImportProviderProps) {
  const merged: TransactionImportState = {
    ...INITIAL_STATE,
    ...initialState,
    accountId: initialAccountId ?? initialState?.accountId ?? null,
  };

  const [state, dispatch] = useReducer(reducer, merged);

  return (
    <TransactionImportContext.Provider value={{ state, dispatch }}>
      {children}
    </TransactionImportContext.Provider>
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// Hook
// ─────────────────────────────────────────────────────────────────────────────

export function useTransactionImport(): TransactionImportContextValue {
  const ctx = useContext(TransactionImportContext);
  if (!ctx) {
    throw new Error("useTransactionImport must be used within a TransactionImportProvider");
  }
  return ctx;
}
