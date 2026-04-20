// Sync-related type definitions

export type SyncStatus = "IDLE" | "RUNNING" | "NEEDS_REVIEW" | "FAILED";

export interface BrokerSyncState {
  accountId: string;
  provider: string;
  checkpointJson?: Record<string, unknown>;
  lastAttemptedAt?: string;
  lastSuccessfulAt?: string;
  lastError?: string;
  lastRunId?: string;
  syncStatus: SyncStatus;
  createdAt: string;
  updatedAt: string;
}

export interface LiabilitiesSection {
  /** Total liabilities value in base currency as decimal string */
  total: string;
  /** Breakdown by individual liability */
  breakdown: import("./activity").BreakdownItem[];
}
