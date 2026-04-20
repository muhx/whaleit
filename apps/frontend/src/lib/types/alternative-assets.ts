// Alternative assets type definitions

export type AlternativeAssetKindApi =
  | "property"
  | "vehicle"
  | "collectible"
  | "precious"
  | "liability"
  | "other";

export interface CreateAlternativeAssetRequest {
  /** The kind of alternative asset */
  kind: AlternativeAssetKindApi;
  /** User-provided name for the asset */
  name: string;
  /** Currency code (e.g., "USD", "EUR") */
  currency: string;
  /** Current total value as decimal string */
  currentValue: string;
  /** Valuation date in ISO format (YYYY-MM-DD) */
  valueDate: string;
  /** Optional purchase price as decimal string - for gain calculation */
  purchasePrice?: string;
  /** Optional purchase date in ISO format */
  purchaseDate?: string;
  /** Kind-specific metadata (e.g., property_type, metal_type, unit) */
  metadata?: Record<string, string>;
  /** For liabilities: optional ID of the financed asset (UI-only linking) */
  linkedAssetId?: string;
}

export interface CreateAlternativeAssetResponse {
  /** Generated asset ID with prefix (e.g., "PROP-a1b2c3d4") */
  assetId: string;
  /** ID of the initial valuation quote */
  quoteId: string;
}

export interface UpdateValuationRequest {
  /** New value as decimal string */
  value: string;
  /** Valuation date in ISO format (YYYY-MM-DD) */
  date: string;
  /** Optional notes about this valuation */
  notes?: string;
}

export interface UpdateValuationResponse {
  /** ID of the created quote */
  quoteId: string;
  /** The valuation date */
  valuationDate: string;
  /** The value as decimal string */
  value: string;
}

export interface PropertyMetadata {
  propertyType?: "residence" | "rental" | "land" | "commercial";
  address?: string;
  purchasePrice?: string;
  purchaseDate?: string;
  purchaseCurrency?: string;
}

export interface VehicleMetadata {
  vehicleType?: "car" | "motorcycle" | "boat" | "rv";
  purchasePrice?: string;
  purchaseDate?: string;
}

export interface CollectibleMetadata {
  collectibleType?: "art" | "wine" | "watch" | "jewelry" | "memorabilia";
  purchasePrice?: string;
  purchaseDate?: string;
}

export interface PreciousMetalMetadata {
  metalType?: "gold" | "silver" | "platinum" | "palladium";
  unit?: "oz" | "g" | "kg";
  purchasePricePerUnit?: string;
  purchaseDate?: string;
}
