// Account-related type definitions

import type { AccountType } from "../constants";
import type { SimplePerformanceMetrics } from "./holding";
import type { TrackingMode } from "./common";

export interface Account {
  id: string;
  name: string;
  accountType: AccountType;
  group?: string; // Optional
  currentBalance?: number; // Decimal serialized as JSON number (rust_decimal serde-float)
  currency: string;
  isDefault: boolean;
  isActive: boolean;
  isArchived: boolean;
  trackingMode: TrackingMode;
  createdAt: Date;
  updatedAt: Date;
  platformId?: string; // Optional - links to platform/broker
  accountNumber?: string; // Optional - account number from broker
  meta?: string; // Optional - additional metadata as JSON string
  provider?: string; // Optional - sync provider (e.g., 'SNAPTRADE', 'PLAID', 'MANUAL')
  providerAccountId?: string; // Optional - account ID in the provider's system
  // Phase 3 additions (D-06, D-11, D-12, D-18):
  institution?: string;
  openingBalance?: number;
  balanceUpdatedAt?: Date;
  creditLimit?: number;
  statementCycleDay?: number;
  statementBalance?: number;
  minimumPayment?: number;
  statementDueDate?: string; // ISO date
  rewardPointsBalance?: number;
  cashbackBalance?: number;
}

export interface AccountSummaryView {
  accountId: string;
  accountName: string;
  accountType: string;
  accountGroup: string | null;
  accountCurrency: string;
  totalValueAccountCurrency: number;
  totalValueBaseCurrency: number;
  baseCurrency: string;
  performance: SimplePerformanceMetrics;
}

export interface AccountGroup {
  groupName: string;
  accounts: AccountSummaryView[];
  totalValueBaseCurrency: number;
  baseCurrency: string;
  performance: SimplePerformanceMetrics;
  accountCount: number;
}

export interface AccountValuation {
  id: string;
  accountId: string;
  valuationDate: string;
  accountCurrency: string;
  baseCurrency: string;
  fxRateToBase: number;
  cashBalance: number;
  investmentMarketValue: number;
  totalValue: number;
  costBasis: number;
  netContribution: number;
  calculatedAt: string;
}

export interface AccountDeposit {
  amount: number;
  currency: string;
  convertedAmount: number;
}
