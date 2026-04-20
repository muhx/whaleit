// Device/platform type definitions

export interface TrackedItem {
  id: string;
  type: "account" | "symbol";
  name: string;
}

// Platform/Broker type
export interface Platform {
  id: string;
  name: string | null;
  url: string;
  externalId: string | null;
  logoUrl?: string | null;
}
