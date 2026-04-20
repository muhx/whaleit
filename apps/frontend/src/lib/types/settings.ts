// Settings-related type definitions

export interface Settings {
  theme: string;
  font: string;
  baseCurrency: string;
  timezone: string;
  instanceId: string;
  onboardingCompleted: boolean;
  autoUpdateCheckEnabled: boolean;
  menuBarVisible: boolean;
  syncEnabled: boolean;
}

export interface SettingsContextType {
  settings: Settings | null;
  isLoading: boolean;
  isError: boolean;
  updateBaseCurrency: (currency: Settings["baseCurrency"]) => Promise<void>;
  accountsGrouped: boolean;
  setAccountsGrouped: (value: boolean) => void;
}

export interface NetWorthConfig {
  includeInvestments: boolean;
  includeProperties: boolean;
  includeVehicles: boolean;
  includeCollectibles: boolean;
  includePreciousMetals: boolean;
  includeOtherAssets: boolean;
  includeLiabilities: boolean;
}
