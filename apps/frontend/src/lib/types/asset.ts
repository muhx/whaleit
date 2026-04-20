// Asset-related type definitions

import type { AssetKind, QuoteMode } from "../constants";
import type { TaxonomyCategory } from "./taxonomy";

export interface Asset {
  id: string;

  // Core identity
  kind: AssetKind;
  name?: string | null;
  displayCode?: string | null;
  notes?: string | null;
  metadata?: Record<string, unknown>;

  // Status
  isActive?: boolean;

  // Valuation
  quoteMode: "MARKET" | "MANUAL";
  quoteCcy: string; // Currency prices/valuations are quoted in

  // Instrument identity (null for non-market assets)
  instrumentType?: string | null; // EQUITY, CRYPTO, FX, OPTION, METAL
  instrumentSymbol?: string | null; // Canonical symbol (AAPL, BTC, EUR)
  instrumentExchangeMic?: string | null; // ISO 10383 MIC (XNAS, XTSE)

  // Computed canonical key (read-only from DB)
  instrumentKey?: string | null;

  // Provider configuration (single JSON blob)
  providerConfig?: Record<string, unknown> | null;

  // Derived
  exchangeName?: string | null; // Friendly exchange name (e.g., "NASDAQ")

  // Audit
  createdAt: string; // ISO date string
  updatedAt: string; // ISO date string
}

export interface NewAsset {
  id?: string;
  kind: string;
  name?: string;
  displayCode?: string;
  isActive: boolean;
  quoteMode: string;
  quoteCcy: string;
  instrumentType?: string;
  instrumentSymbol?: string;
  instrumentExchangeMic?: string;
  notes?: string;
}

export interface AssetClassifications {
  assetType?: TaxonomyCategory | null;
  riskCategory?: TaxonomyCategory | null;
  assetClasses: import("./taxonomy").CategoryWithWeight[];
  sectors: import("./taxonomy").CategoryWithWeight[];
  regions: import("./taxonomy").CategoryWithWeight[];
  customGroups: import("./taxonomy").CategoryWithWeight[];
}

export interface AssetTaxonomyAssignment {
  id: string;
  assetId: string;
  taxonomyId: string;
  categoryId: string;
  weight: number; // basis points: 10000 = 100%
  source: string; // "manual", "provider", "inferred"
  createdAt: string;
  updatedAt: string;
}

export interface NewAssetTaxonomyAssignment {
  id?: string | null;
  assetId: string;
  taxonomyId: string;
  categoryId: string;
  weight: number; // basis points: 10000 = 100%
  source: string;
}

export interface UpdateAssetProfile {
  id: string;
  displayCode?: string | null;
  name?: string | null;
  notes?: string | null;
  kind?: AssetKind | null;
  quoteMode?: QuoteMode | null;
  quoteCcy?: string | null;
  instrumentType?: string | null;
  instrumentExchangeMic?: string | null;
  providerConfig?: Record<string, unknown> | null;
}

export interface StaleAssetInfo {
  /** Asset ID */
  assetId: string;
  /** Asset name (if available) */
  name?: string;
  /** Date of the last valuation (ISO format) */
  valuationDate: string;
  /** Number of days since last valuation */
  daysStale: number;
}

export interface SymbolSearchResult {
  exchange: string;
  /** Canonical exchange MIC code (e.g., "XNAS", "XTSE") */
  exchangeMic?: string;
  /** Friendly exchange name (e.g., "NASDAQ" instead of "NMS" or "XNAS") */
  exchangeName?: string;
  /** Currency derived from exchange (e.g., "USD", "CAD") */
  currency?: string;
  /** Provenance: "provider" | "exchange_inferred" */
  currencySource?: string;
  shortName: string;
  quoteType: string;
  symbol: string;
  index: string;
  score: number;
  typeDisplay: string;
  longName: string;
  dataSource?: string;
  /** Asset kind for custom assets (e.g., "SECURITY", "CRYPTO", "OTHER") */
  assetKind?: string;
  /** True if this asset already exists in user's database */
  isExisting?: boolean;
  /** The existing asset ID if found (e.g., "SEC:AAPL:XNAS") */
  existingAssetId?: string;
}
