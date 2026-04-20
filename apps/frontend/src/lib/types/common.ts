// Common type definitions

export interface UpdateInfo {
  currentVersion: string;
  latestVersion: string;
  notes?: string;
  pubDate?: string;
  isAppStoreBuild: boolean;
  storeUrl?: string;
  changelogUrl?: string;
  screenshots?: string[];
}

export type ValidationResult = { status: "success" } | { status: "error"; errors: string[] };

/**
 * Tracking mode for an account - determines how holdings are tracked.
 * Matches the backend TrackingMode enum.
 */
export type TrackingMode = "TRANSACTIONS" | "HOLDINGS" | "NOT_SET";

// Define custom DateRange type matching react-day-picker's
export interface DateRange {
  from: Date | undefined;
  to: Date | undefined;
}

export type TimePeriod = "1D" | "1W" | "1M" | "3M" | "6M" | "YTD" | "1Y" | "5Y" | "ALL";

// Addon Store Types
export interface AddonStoreListing {
  id: string;
  name: string;
  version: string;
  description: string;
  author: string;
  downloadUrl: string;
  downloads: number;
  rating: number;
  reviewCount: number;
  status?: "active" | "inactive" | "deprecated" | "coming-soon";
  lastUpdated: string;
  releaseNotes: string;
  changelogUrl: string;
  images: string[];
  /** Classification tags for filtering */
  tags?: string[];
}
