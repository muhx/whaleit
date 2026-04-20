// Activity-related type definitions

import { importActivitySchema, importMappingSchema, parseConfigSchema } from "@/lib/schemas";
import * as z from "zod";
import type { QuoteMode, ActivityStatus, ActivityType } from "../constants";
import { ACTIVITY_TYPE_DISPLAY_NAMES, SUBTYPE_DISPLAY_NAMES } from "../constants";

export interface Activity {
  // Identity
  id: string;
  accountId: string;
  assetId?: string; // NOW OPTIONAL for pure cash events

  // Classification
  activityType: string; // Canonical type (closed set of 15)
  activityTypeOverride?: string; // User override (never touched by sync)
  sourceType?: string; // Raw provider label (REI, DIV, etc.)
  subtype?: string; // Semantic variation (DRIP, STAKING_REWARD, etc.)
  status: ActivityStatus;

  // Timing
  activityDate: string; // ISO timestamp (UTC)
  settlementDate?: string;

  // Quantities (strings to preserve decimal precision)
  quantity?: string | null;
  unitPrice?: string | null;
  amount?: string | null;
  fee?: string | null;
  currency: string;
  fxRate?: string | null;

  // Metadata
  notes?: string;
  metadata?: Record<string, unknown>;

  // Source identity
  sourceSystem?: string; // SNAPTRADE, PLAID, MANUAL, CSV
  sourceRecordId?: string;
  sourceGroupId?: string;
  idempotencyKey?: string;
  importRunId?: string;

  // Sync flags
  isUserModified: boolean; // User edited; sync protects economics
  needsReview: boolean; // Needs user review (low confidence, etc.)

  // Audit
  createdAt: string;
  updatedAt: string;
}

/**
 * Helper to get effective type (respects user override)
 */
export function getEffectiveType(activity: Activity): string {
  return activity.activityTypeOverride ?? activity.activityType;
}

/**
 * Check if activity has user override
 */
export function hasUserOverride(activity: Activity): boolean {
  return activity.activityTypeOverride !== undefined && activity.activityTypeOverride !== null;
}

/**
 * Get display name for an activity
 */
export function getActivityDisplayName(activity: Activity): string {
  // Check subtype first (most specific)
  if (activity.subtype && SUBTYPE_DISPLAY_NAMES[activity.subtype]) {
    return SUBTYPE_DISPLAY_NAMES[activity.subtype];
  }
  // Use effective type (respects user override)
  const effectiveType = getEffectiveType(activity);
  return (ACTIVITY_TYPE_DISPLAY_NAMES as Record<string, string>)[effectiveType] || effectiveType;
}

export interface ActivityDetails {
  id: string;
  activityType: ActivityType;
  subtype?: string | null;
  status?: ActivityStatus;
  date: Date;
  quantity: string | null;
  unitPrice: string | null;
  amount: string | null;
  fee: string | null;
  currency: string;
  needsReview: boolean;
  comment?: string;
  fxRate?: string | null;
  createdAt: Date;
  assetId: string;
  updatedAt: Date;
  accountId: string;
  accountName: string;
  accountCurrency: string;
  assetSymbol: string;
  assetName?: string;
  assetQuoteMode?: QuoteMode;
  /** Canonical exchange MIC code for asset identification */
  exchangeMic?: string;
  instrumentType?: string;
  // Sync/source metadata
  sourceSystem?: string;
  sourceRecordId?: string;
  idempotencyKey?: string;
  importRunId?: string;
  isUserModified?: boolean;
  metadata?: Record<string, unknown>;
  subRows?: ActivityDetails[];
}

export interface ActivitySearchResponse {
  data: ActivityDetails[];
  meta: {
    totalRowCount: number;
  };
}

/**
 * Symbol input for creating/updating activities.
 * Groups all symbol-related fields into a single nested object.
 */
export interface SymbolInput {
  id?: string; // Only for updates (backend generates ID for creates)
  symbol?: string; // e.g., "AAPL" or undefined for cash
  exchangeMic?: string; // e.g., "XNAS" or undefined
  kind?: string; // e.g., "INVESTMENT", "OTHER" - asset kind
  name?: string; // Asset name (for custom assets)
  quoteMode?: QuoteMode;
  quoteCcy?: string; // Optional quote currency hint from search/provider (e.g., "GBp")
  instrumentType?: string; // Optional instrument type hint (e.g., "EQUITY", "CRYPTO")
}

export interface ActivityCreate {
  id?: string;
  idempotencyKey?: string;
  accountId: string;
  activityType: string;
  subtype?: string | null; // Semantic variation (DRIP, STAKING_REWARD, etc.)
  activityDate: string | Date;
  /** Optional grouping key (links paired transfer legs). */
  sourceGroupId?: string;
  symbol?: SymbolInput;
  quantity?: string | number | null;
  unitPrice?: string | number | null;
  amount?: string | number | null;
  currency?: string;
  fee?: string | number | null;
  comment?: string | null;
  fxRate?: string | number | null;
  metadata?: string | Record<string, unknown>; // Metadata (serialized to JSON string before sending)
}

export interface ActivityUpdate {
  id: string;
  accountId: string;
  activityType: string;
  subtype?: string | null;
  activityDate: string | Date;
  /** Optional grouping key (links paired transfer legs). */
  sourceGroupId?: string;
  symbol?: SymbolInput;
  quantity?: string | number | null;
  unitPrice?: string | number | null;
  amount?: string | number | null;
  currency?: string;
  fee?: string | number | null;
  comment?: string | null;
  fxRate?: string | number | null;
  metadata?: string | Record<string, unknown>; // Metadata (serialized to JSON string before sending)
}

export interface ActivityBulkMutationRequest {
  creates?: ActivityCreate[];
  updates?: ActivityUpdate[];
  deleteIds?: string[];
}
export interface ActivityBulkMutationError {
  id?: string;
  action: string;
  message: string;
}
export interface ActivityBulkIdentifierMapping {
  tempId?: string | null;
  activityId: string;
}
export interface ActivityBulkMutationResult {
  created: Activity[];
  updated: Activity[];
  deleted: Activity[];
  createdMappings: ActivityBulkIdentifierMapping[];
  errors: ActivityBulkMutationError[];
}
export type ActivityImport = z.infer<typeof importActivitySchema>;
export type ImportMappingData = z.infer<typeof importMappingSchema>;
export type ParseConfig = z.infer<typeof parseConfigSchema>;
export type ImportTemplateScope = "SYSTEM" | "USER";

export interface ImportTemplateData {
  id: string;
  name: string;
  scope: ImportTemplateScope;
  kind: TemplateKind;
  fieldMappings: Record<string, string | string[]>;
  activityMappings: Record<string, string[]>;
  symbolMappings: Record<string, string>;
  accountMappings: Record<string, string>;
  symbolMappingMeta: Record<
    string,
    {
      exchangeMic?: string;
      symbolName?: string;
      quoteCcy?: string;
      instrumentType?: string;
      quoteMode?: QuoteMode;
    }
  >;
  parseConfig?: ParseConfig;
}

export type TemplateKind = "CSV_ACTIVITY" | "CSV_HOLDINGS" | "BROKER_ACTIVITY";
export type TemplateContextKind = TemplateKind;

export type BrokerProfileScope = "ACCOUNT" | "BROKER";

export interface BrokerSyncProfileData {
  id: string;
  name: string;
  scope: ImportTemplateScope;
  sourceSystem: string;
  activityMappings: Record<string, string[]>;
  symbolMappings: Record<string, string>;
  symbolMappingMeta: Record<
    string,
    {
      exchangeMic?: string;
      symbolName?: string;
      quoteCcy?: string;
      instrumentType?: string;
    }
  >;
}

export interface SaveBrokerSyncProfileRulesRequest {
  accountId: string;
  sourceSystem: string;
  scope: BrokerProfileScope;
  activityRulePatches: Record<string, string[]>;
  securityRulePatches: Record<string, string>;
  securityRuleMetaPatches: Record<
    string,
    {
      exchangeMic?: string;
      symbolName?: string;
      quoteCcy?: string;
      instrumentType?: string;
    }
  >;
}

// Define a generic type for the parsed row data
export type CsvRowData = Record<string, string> & { lineNumber: string };
export interface CsvRowError {
  /** Type of error that occurred */
  type: string;
  /** Standardized error code */
  code: string;
  /** Human-readable error message */
  message: string;
  /** Row index where the error occurred (optional) */
  row?: number;
  /** Column/field index where the error occurred (optional) */
  index?: number;
}

/**
 * Error encountered during CSV parsing.
 */
export interface ParseError {
  /** Row index where the error occurred (if applicable) */
  rowIndex?: number;
  /** Column index where the error occurred (if applicable) */
  columnIndex?: number;
  /** Human-readable error message */
  message: string;
  /** Error type: "parse", "encoding", "structure" */
  errorType: string;
}

/**
 * Result of parsing a CSV file.
 */
export interface ParsedCsvResult {
  /** Headers extracted from the CSV */
  headers: string[];
  /** Data rows (each row is an array of string values) */
  rows: string[][];
  /** The configuration values actually used (with auto-detected values filled in) */
  detectedConfig: ParseConfig;
  /** Any errors encountered during parsing */
  errors: ParseError[];
  /** Total number of data rows (excluding headers and skipped rows) */
  rowCount: number;
}

export interface ImportValidationResult {
  activities: ActivityImport[];
  validationSummary: {
    totalRows: number;
    validCount: number;
    invalidCount: number;
  };
}

/**
 * Result of importing activities, includes import run metadata
 */
export interface ImportActivitiesResult {
  /** The validated/imported activities */
  activities: ActivityImport[];
  /** Import run ID for tracking this batch */
  importRunId: string;
  /** Summary statistics for the import */
  summary: ImportActivitiesSummary;
}

/**
 * Summary statistics for an activity import
 */
export interface ImportActivitiesSummary {
  /** Total number of activities in the import request */
  total: number;
  /** Number of activities successfully imported */
  imported: number;
  /** Number of activities skipped (invalid or errors) */
  skipped: number;
  /** Number of duplicate activities detected and skipped */
  duplicates: number;
  /** Number of new assets created during import */
  assetsCreated: number;
  /** Whether the import was successful (no validation errors) */
  success: boolean;
  /** Human-readable reason for failure, if success is false */
  errorMessage?: string;
}

// Import Run types
export type ImportRunType = "SYNC" | "IMPORT";
export type ImportRunMode = "INITIAL" | "INCREMENTAL" | "BACKFILL" | "REPAIR";
export type ImportRunStatus = "RUNNING" | "APPLIED" | "NEEDS_REVIEW" | "FAILED" | "CANCELLED";
export type ReviewMode = "NEVER" | "ALWAYS" | "IF_WARNINGS";

export interface ImportRunSummary {
  fetched: number;
  inserted: number;
  updated: number;
  skipped: number;
  warnings: number;
  errors: number;
  removed: number;
}

export interface ImportRun {
  id: string;
  accountId: string;
  sourceSystem: string;
  runType: ImportRunType;
  mode: ImportRunMode;
  status: ImportRunStatus;
  startedAt: string;
  finishedAt?: string;
  reviewMode: ReviewMode;
  appliedAt?: string;
  checkpointIn?: Record<string, unknown>;
  checkpointOut?: Record<string, unknown>;
  summary?: ImportRunSummary;
  warnings?: string[];
  error?: string;
  createdAt: string;
  updatedAt: string;
}

export interface ImportAssetCandidate {
  key: string;
  accountId: string;
  symbol: string;
  currency?: string;
  instrumentType?: string;
  quoteCcy?: string;
  quoteMode?: string;
  exchangeMic?: string;
  isin?: string;
}

export type ImportAssetPreviewStatus =
  | "EXISTING_ASSET"
  | "AUTO_RESOLVED_NEW_ASSET"
  | "NEEDS_FIXING";

export interface ImportAssetPreviewItem {
  key: string;
  status: ImportAssetPreviewStatus;
  resolutionSource: string;
  assetId?: string;
  draft?: import("./asset").NewAsset;
  errors?: Record<string, string[]>;
  warnings?: Record<string, string[]>;
}

export interface BreakdownItem {
  /** Category key (e.g., "cash", "investments", "properties") */
  category: string;
  /** Display name */
  name: string;
  /** Value in base currency (positive magnitude) as decimal string */
  value: string;
  /** Optional: asset ID for individual items */
  assetId?: string;
}

/**
 * Assets section of the balance sheet
 */
export interface AssetsSection {
  /** Total assets value in base currency as decimal string */
  total: string;
  /** Breakdown by category */
  breakdown: BreakdownItem[];
}
