// Contribution limits type definitions

export interface ContributionLimit {
  id: string;
  groupName: string;
  contributionYear: number;
  limitAmount: number;
  accountIds?: string | null;
  startDate?: string | null;
  endDate?: string | null;
  createdAt?: string;
  updatedAt?: string;
}

export type NewContributionLimit = Omit<ContributionLimit, "id" | "createdAt" | "updatedAt">;

export interface DepositsCalculation {
  total: number;
  baseCurrency: string;
  byAccount: Record<string, import("./account").AccountDeposit>;
}
