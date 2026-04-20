// Holding-related type definitions

import type { AssetKind, HoldingType, QuoteMode } from "../constants";
import type { AssetClassifications } from "./asset";

export interface Instrument {
  id: string;
  symbol: string;
  name?: string | null;
  currency: string;
  notes?: string | null;
  quoteMode: QuoteMode;
  preferredProvider?: string | null;

  // Taxonomy-based classifications
  classifications?: AssetClassifications | null;
}

export interface MonetaryValue {
  local: number;
  base: number;
}

export interface Lot {
  id: string;
  positionId: string;
  acquisitionDate: string; // ISO date string
  quantity: number;
  costBasis: number;
  acquisitionPrice: number;
  acquisitionFees: number;
}

export interface Position {
  id: string;
  accountId: string;
  assetId: string;
  quantity: number;
  averageCost: number;
  totalCostBasis: number;
  currency: string;
  inceptionDate: string; // ISO date string
  lots: Lot[];
}

export interface CashHolding {
  id: string;
  accountId: string;
  currency: string;
  amount: number;
  lastUpdated: string; // ISO date string
}

export interface Holding {
  id: string;
  holdingType: HoldingType;
  accountId: string;
  instrument?: Instrument | null;
  assetKind?: AssetKind | null;
  quantity: number;
  openDate?: string | Date | null;
  lots?: Lot[] | null;
  localCurrency: string;
  baseCurrency: string;
  fxRate?: number | null;
  marketValue: MonetaryValue;
  costBasis?: MonetaryValue | null;
  price?: number | null;
  unrealizedGain?: MonetaryValue | null;
  unrealizedGainPct?: number | null;
  realizedGain?: MonetaryValue | null;
  realizedGainPct?: number | null;
  totalGain?: MonetaryValue | null;
  totalGainPct?: number | null;
  dayChange?: MonetaryValue | null;
  dayChangePct?: number | null;
  prevCloseValue?: MonetaryValue | null;
  weight: number;
  asOfDate: string;
}

export interface HoldingSummary {
  id: string;
  symbol: string;
  name?: string | null;
  holdingType: HoldingType;
  quantity: number;
  marketValue: number; // Base currency value
  currency: string;
  weightInCategory: number; // Percentage weight within the category (0-100)
}

export interface AllocationHoldings {
  taxonomyId: string;
  taxonomyName: string;
  categoryId: string;
  categoryName: string;
  color: string;
  holdings: HoldingSummary[];
  totalValue: number;
  currency: string;
}

export interface SimplePerformanceMetrics {
  accountId: string;
  totalValue?: number | null;
  accountCurrency?: string | null;
  baseCurrency?: string | null;
  fxRateToBase?: number | null;
  totalGainLossAmount?: number | null;
  cumulativeReturnPercent?: number | null;
  dayGainLossAmount?: number | null;
  dayReturnPercentModDietz?: number | null;
  portfolioWeight?: number | null;
}

// Renamed from CumulativeReturn to match Rust struct ReturnData
export interface ReturnData {
  date: string; // Changed from CumulativeReturn
  value: number;
}

// Renamed from PerformanceData to match Rust struct
export interface PerformanceMetrics {
  id: string;
  returns: ReturnData[];
  periodStartDate?: string | null;
  periodEndDate?: string | null;
  currency: string;
  /** Period gain in dollars (SOTA: change in unrealized P&L for HOLDINGS mode) */
  periodGain: number;
  /** Period return percentage (SOTA formula for HOLDINGS mode). Null when start value ≤ 0. */
  periodReturn: number | null;
  /** Time-weighted return (null for HOLDINGS mode - requires cash flow tracking) */
  cumulativeTwr?: number | null;
  /** Legacy field for backward compatibility */
  gainLossAmount?: number | null;
  /** Annualized TWR (null for HOLDINGS mode) */
  annualizedTwr?: number | null;
  simpleReturn: number;
  annualizedSimpleReturn: number;
  /** Money-weighted return (null for HOLDINGS mode - requires cash flow tracking) */
  cumulativeMwr?: number | null;
  /** Annualized MWR (null for HOLDINGS mode) */
  annualizedMwr?: number | null;
  volatility: number;
  maxDrawdown: number;
  /** Indicates if this is a HOLDINGS mode account (no cash flow tracking) */
  isHoldingsMode?: boolean;
}

export interface AlternativeAssetHolding {
  /** Asset ID (e.g., "PROP-a1b2c3d4") */
  id: string;
  /** Asset kind (property, vehicle, collectible, precious, liability, other) */
  kind: string;
  /** Asset name */
  name: string;
  /** Asset symbol (display type label, e.g., "Property", "Vehicle") */
  symbol: string;
  /** Currency */
  currency: string;
  /** Current market value from latest quote */
  marketValue: string;
  /** Purchase price if available (from metadata) */
  purchasePrice?: string;
  /** Purchase date if available (from metadata) */
  purchaseDate?: string;
  /** Unrealized gain (market_value - purchase_price) */
  unrealizedGain?: string;
  /** Unrealized gain percentage */
  unrealizedGainPct?: string;
  /** Date of the latest valuation (ISO format) */
  valuationDate: string;
  /** Kind-specific metadata */
  metadata?: Record<string, unknown>;
  /** For liabilities: linked asset ID if any */
  linkedAssetId?: string;
  /** Asset notes */
  notes?: string | null;
}

export interface SnapshotInfo {
  /** Snapshot ID */
  id: string;
  /** Date of the snapshot (YYYY-MM-DD) */
  snapshotDate: string;
  /** Source of the snapshot (MANUAL_ENTRY, CSV_IMPORT, BROKER_IMPORTED) */
  source: string;
  /** Number of positions in this snapshot */
  positionCount: number;
  /** Number of cash currencies in this snapshot */
  cashCurrencyCount: number;
}

export interface HoldingsPositionInput {
  /** Symbol from CSV (e.g., "AAPL", "GOOGL") */
  symbol: string;
  /** Quantity held as string to preserve precision */
  quantity: string;
  /** Optional average cost per unit */
  avgCost?: string;
  /** Currency for this position */
  currency: string;
  /** Exchange MIC code (e.g., "XNAS", "XTSE") resolved during check step */
  exchangeMic?: string;
  /** Resolved asset ID from asset review step */
  assetId?: string;
}

export interface HoldingsSnapshotInput {
  /** The date of this snapshot (YYYY-MM-DD) */
  date: string;
  /** Securities held on this date */
  positions: HoldingsPositionInput[];
  /** Cash balances by currency (e.g., {"USD": "10000", "EUR": "5000"}) */
  cashBalances: Record<string, string>;
}

export interface ImportHoldingsCsvResult {
  /** Number of snapshots successfully imported */
  snapshotsImported: number;
  /** Number of snapshots that failed to import */
  snapshotsFailed: number;
  /** Error messages for failed snapshots */
  errors: string[];
}

export interface SymbolCheckResult {
  symbol: string;
  found: boolean;
  assetName?: string;
  assetId?: string;
  currency?: string;
  exchangeMic?: string;
}

export interface CheckHoldingsImportResult {
  /** Dates that already have snapshots in the DB (will be overwritten) */
  existingDates: string[];
  /** Per-unique-symbol lookup results */
  symbols: SymbolCheckResult[];
  /** Validation errors found in the import data */
  validationErrors: string[];
}
