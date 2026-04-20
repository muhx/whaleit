// Barrel re-export — backward compatible with all existing imports
// import { X } from "@/lib/types" continues to work

// Re-export from schemas (preserving existing re-exports)
export { ImportType } from "./schemas";

// Re-export constants that were previously re-exported from types.ts
export {
  AccountType,
  ActivityStatus,
  ActivityType,
  ACTIVITY_SUBTYPES,
  ACTIVITY_TYPE_DISPLAY_NAMES,
  ACTIVITY_TYPES,
  AlternativeAssetKind,
  ALTERNATIVE_ASSET_DEFAULT_GROUPS,
  ALTERNATIVE_ASSET_KIND_DISPLAY_NAMES,
  AssetKind,
  ASSET_KIND_DISPLAY_NAMES,
  createPortfolioAccount,
  DataSource,
  defaultGroupForAccountType,
  ExportDataType,
  ExportedFileFormat,
  HOLDING_CATEGORY_FILTERS,
  HOLDING_GROUP_DISPLAY_NAMES,
  HOLDING_GROUP_ORDER,
  HoldingType,
  ImportFormat,
  PricingMode,
  QuoteMode,
  SUBTYPE_DISPLAY_NAMES,
} from "./constants";

export type { HoldingCategoryFilterId } from "./constants";
export type { ActivitySubtype, ImportRequiredField } from "./constants";

// Domain type files
export * from "./types/account";
export * from "./types/activity";
export * from "./types/asset";
export * from "./types/holding";
export * from "./types/portfolio";
export * from "./types/quote";
export * from "./types/goal";
export * from "./types/settings";
export * from "./types/taxonomy";
export * from "./types/health";
export * from "./types/ai";
export * from "./types/sync";
export * from "./types/device";
export * from "./types/alternative-assets";
export * from "./types/contributions";
export * from "./types/liabilities";
export * from "./types/net-worth";
export * from "./types/fx";
export * from "./types/common";
export * from "./types/tag";
