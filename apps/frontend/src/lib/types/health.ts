// Health center type definitions

export type HealthSeverity = "INFO" | "WARNING" | "ERROR" | "CRITICAL";

export type HealthCategory =
  | "PRICE_STALENESS"
  | "FX_INTEGRITY"
  | "CLASSIFICATION"
  | "DATA_CONSISTENCY"
  | "ACCOUNT_CONFIGURATION"
  | "SETTINGS_CONFIGURATION";

export interface NavigateAction {
  route: string;
  query?: Record<string, unknown>;
  label: string;
}

export interface FixAction {
  id: string;
  label: string;
  payload: Record<string, unknown>;
}

export interface AffectedItem {
  id: string;
  name: string;
  symbol?: string;
  route?: string;
}

export interface HealthIssue {
  id: string;
  severity: HealthSeverity;
  category: HealthCategory;
  title: string;
  message: string;
  affectedCount: number;
  affectedMvPct?: number;
  fixAction?: FixAction;
  navigateAction?: NavigateAction;
  details?: string;
  affectedItems?: AffectedItem[];
  dataHash: string;
  timestamp: string;
}

export interface HealthStatus {
  overallSeverity: HealthSeverity;
  issueCounts: Partial<Record<HealthSeverity, number>>;
  issues: HealthIssue[];
  checkedAt: string;
  isStale: boolean;
}

export interface HealthConfig {
  stalePriceWarningDays: number;
  stalePriceErrorDays: number;
  criticalMvThresholdPercent: number;
  enabled: boolean;
}
