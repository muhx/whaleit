// Quote-related type definitions

export interface Quote {
  id: string;
  createdAt: string;
  dataSource: string;
  timestamp: string;
  assetId: string;
  open: number;
  high: number;
  low: number;
  volume: number;
  close: number;
  adjclose: number;
  currency: string;
  notes?: string | null;
}

export interface QuoteUpdate {
  timestamp: string;
  assetId: string;
  open: number;
  high: number;
  low: number;
  volume: number;
  close: number;
  dataSource: string;
}

export interface LatestQuoteSnapshot {
  quote: Quote;
  isStale: boolean;
  effectiveMarketDate: string; // YYYY-MM-DD in market timezone semantics
  quoteDate: string; // YYYY-MM-DD extracted from quote timestamp
}

export interface ResolvedQuote {
  currency?: string;
  price?: number;
  resolvedProviderId?: string;
}

export interface ExchangeInfo {
  mic: string;
  name: string;
  longName: string;
  currency: string;
}

export interface MarketDataProviderInfo {
  id: string;
  name: string;
  logoFilename: string;
  lastSyncedDate: string | null; // ISO date string
  providerType?: string;
}

export interface MarketData {
  createdAt: Date;
  dataSource: string;
  date: Date;
  id: string;
  marketPrice: number;
  state: string; // backend-provided market state; do not assume only "CLOSE"
  symbol: string;
  symbolProfileId: string;
}
