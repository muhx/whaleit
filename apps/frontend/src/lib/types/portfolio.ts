// Portfolio-related type definitions

import type { AssetKind } from "../constants";
import type { TaxonomyAllocation } from "./taxonomy";

export interface PortfolioAllocations {
  assetClasses: TaxonomyAllocation;
  sectors: TaxonomyAllocation;
  regions: TaxonomyAllocation;
  riskCategory: TaxonomyAllocation;
  securityTypes: TaxonomyAllocation;
  customGroups: TaxonomyAllocation[];
  totalValue: number;
}

export interface IncomeByAsset {
  assetId: string;
  kind: AssetKind;
  symbol: string;
  name: string;
  income: number;
}

export interface IncomeByAccount {
  accountId: string;
  accountName: string;
  byMonth: Record<string, number>;
  total: number;
}

export interface IncomeSummary {
  period: string;
  byMonth: Record<string, number>;
  byType: Record<string, number>;
  byAsset: Record<string, IncomeByAsset>;
  byCurrency: Record<string, number>;
  byAccount: Record<string, IncomeByAccount>;
  totalIncome: number;
  currency: string;
  monthlyAverage: number;
  yoyGrowth: number | null; // Changed from optional to nullable
}
