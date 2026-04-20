// Net worth type definitions

import type { StaleAssetInfo } from "./asset";
import type { AssetsSection } from "./activity";
import type { LiabilitiesSection } from "./sync";

export interface NetWorthResponse {
  /** As-of date for the calculation (ISO format) */
  date: string;
  /** Assets section with total and breakdown */
  assets: AssetsSection;
  /** Liabilities section with total and breakdown */
  liabilities: LiabilitiesSection;
  /** Net worth (assets - liabilities) as decimal string */
  netWorth: string;
  /** Base currency used for the calculation */
  currency: string;
  /** Oldest valuation date used in the calculation */
  oldestValuationDate?: string;
  /** Assets with valuations older than 90 days */
  staleAssets: StaleAssetInfo[];
}

export interface NetWorthHistoryPoint {
  /** Date of this data point (ISO format) */
  date: string;

  // Component values
  /** Portfolio value from TOTAL account (investments + cash) as decimal string */
  portfolioValue: string;
  /** Alternative assets value (properties, vehicles, collectibles, etc.) as decimal string */
  alternativeAssetsValue: string;
  /** Total liabilities as decimal string (positive magnitude, subtracted for net worth) */
  totalLiabilities: string;

  // Totals
  /** Total assets = portfolio_value + alternative_assets_value as decimal string */
  totalAssets: string;
  /** Net worth (assets - liabilities) as decimal string */
  netWorth: string;

  // For gain calculation
  /** Cumulative net contributions (deposits - withdrawals) from portfolio as decimal string */
  netContribution: string;

  /** Currency */
  currency: string;
}
