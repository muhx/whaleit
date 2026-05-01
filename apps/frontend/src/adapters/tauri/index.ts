import type { RunEnv } from "../types";
import { RunEnvs } from "../types";

export { logger } from "./core";
export { isDesktop, isWeb } from "./core";

export const RUN_ENV: RunEnv = RunEnvs.DESKTOP;

export type { EventCallback, UnlistenFn, RunEnv, Logger } from "../types";
export { RunEnvs } from "../types";
export type {
  AddonFile,
  AddonInstallResult,
  AddonManifest,
  AddonUpdateCheckResult,
  AddonUpdateInfo,
  AddonValidationResult,
  AppInfo,
  BackendEnableSyncResult,
  BackendSyncBackgroundEngineResult,
  BackendSyncBootstrapOverwriteCheckResult,
  BackendSyncBootstrapResult,
  BackendSyncCycleResult,
  BackendSyncEngineStatusResult,
  BackendSyncReconcileReadyResult,
  BackendSyncSnapshotUploadResult,
  BackendSyncStateResult,
  EphemeralKeyPair,
  ExtractedAddon,
  FunctionPermission,
  ImportRunsRequest,
  InstalledAddon,
  MarketDataProviderSetting,
  Permission,
  PlatformCapabilities,
  PlatformInfo,
  ProviderCapabilities,
  UpdateCheckPayload,
  UpdateCheckResult,
  UpdateThreadRequest,
  UpdateToolResultRequest,
} from "../types";

export type {
  AiChatMessage,
  AiChatModelConfig,
  AiSendMessageRequest,
  AiStreamEvent,
  AiThread,
  AiToolCall,
  AiToolResult,
  AiUsageStats,
  ListThreadsRequest,
  ThreadPage,
} from "@/features/ai-assistant/types";

export { createAccount, deleteAccount, getAccounts, updateAccount } from "../shared/accounts";

export {
  checkActivitiesImport,
  checkExistingDuplicates,
  createActivity,
  deleteImportTemplate,
  deleteActivity,
  getImportTemplate,
  getAccountImportMapping,
  linkAccountTemplate,
  getActivities,
  importActivities,
  listImportTemplates,
  previewImportAssets,
  saveAccountImportMapping,
  saveImportTemplate,
  saveActivities,
  searchActivities,
  updateActivity,
} from "../shared/activities";
export { parseCsv } from "../web/activities";

// Transaction Commands (Phase 4)
export {
  breakTransferPair,
  createTransaction,
  createTransfer,
  deleteTransaction,
  deleteTransactionTemplate,
  detectTransactionDuplicates,
  getAccountRecentTransactions,
  getTransaction,
  getTransactionTemplate,
  listPayeeCategoryMemory,
  listRunningBalance,
  listTransactionTemplates,
  lookupPayeeCategory,
  previewTransactionImport,
  saveTransactionTemplate,
  searchTransactions,
  updateTransaction,
  updateTransferLeg,
} from "../shared/transactions";

export {
  createGoal,
  deleteGoal,
  getGoals,
  getGoalsAllocation,
  updateGoal,
  updateGoalsAllocations,
} from "../shared/goals";

export { deleteSecret, getSecret, setSecret } from "../shared/secrets";

export {
  assignAssetToCategory,
  createCategory,
  createTaxonomy,
  deleteCategory,
  deleteTaxonomy,
  exportTaxonomyJson,
  getAssetTaxonomyAssignments,
  getMigrationStatus,
  getTaxonomies,
  getTaxonomy,
  importTaxonomyJson,
  migrateLegacyClassifications,
  moveCategory,
  removeAssetTaxonomyAssignment,
  updateCategory,
  updateTaxonomy,
} from "../shared/taxonomies";

export {
  calculateAccountsSimplePerformance,
  calculatePerformanceHistory,
  calculatePerformanceSummary,
  checkHoldingsImport,
  deleteSnapshot,
  getAssetHoldings,
  getHistoricalValuations,
  getHolding,
  getHoldings,
  getHoldingsByAllocation,
  getIncomeSummary,
  getLatestValuations,
  getPortfolioAllocations,
  getSnapshotByDate,
  getSnapshots,
  importHoldingsCsv,
  recalculatePortfolio,
  saveManualHoldings,
  updatePortfolio,
} from "../shared/portfolio";

export type { HoldingInput } from "../shared/portfolio";

export {
  checkQuotesImport,
  createAsset,
  deleteAsset,
  deleteQuote,
  fetchYahooDividends,
  getAssetProfile,
  getAssets,
  getExchanges,
  getLatestQuotes,
  getMarketDataProviders,
  getMarketDataProviderSettings,
  getQuoteHistory,
  importManualQuotes,
  resolveSymbolQuote,
  searchTicker,
  syncHistoryQuotes,
  syncMarketData,
  updateAssetProfile,
  updateMarketDataProviderSettings,
  updateQuote,
  updateQuoteMode,
} from "../shared/market-data";

export {
  getCustomProviders,
  createCustomProvider,
  updateCustomProvider,
  deleteCustomProvider,
  testCustomProviderSource,
} from "../shared/custom-provider";

export {
  calculateDepositsForLimit,
  createContributionLimit,
  deleteContributionLimit,
  getContributionLimit,
  updateContributionLimit,
} from "../shared/contribution-limits";

export {
  addExchangeRate,
  deleteExchangeRate,
  getExchangeRates,
  updateExchangeRate,
} from "../shared/exchange-rates";

export {
  createAlternativeAsset,
  deleteAlternativeAsset,
  getAlternativeHoldings,
  getNetWorth,
  getNetWorthHistory,
  linkLiability,
  unlinkLiability,
  updateAlternativeAssetMetadata,
  updateAlternativeAssetValuation,
} from "../shared/alternative-assets";

export {
  approvePairing,
  approvePairingOverwrite,
  beginPairingConfirm,
  cancelPairing,
  cancelPairingFlow,
  claimPairing,
  clearDeviceSyncData,
  clearSyncSession,
  completePairing,
  completePairingWithTransfer,
  confirmPairing,
  confirmPairingWithBootstrap,
  createPairing,
  getPairingFlowState,
  deleteDevice,
  deviceSyncBootstrapOverwriteCheck,
  deviceSyncCancelSnapshotUpload,
  deviceSyncGenerateSnapshotNow,
  deviceSyncReconcileReadyState,
  deviceSyncStartBackgroundEngine,
  deviceSyncStopBackgroundEngine,
  enableDeviceSync,
  getBrokerSyncStates,
  getDevice,
  getDeviceSyncState,
  getImportRuns,
  getPairingSourceStatus,
  getPairing,
  getPairingMessages,
  getPlatforms,
  getSubscriptionPlans,
  getSubscriptionPlansPublic,
  getSyncedAccounts,
  getSyncEngineStatus,
  getUserInfo,
  listBrokerAccounts,
  listBrokerConnections,
  listDevices,
  reinitializeDeviceSync,
  resetTeamSync,
  restoreSyncSession,
  revokeDevice,
  storeSyncSession,
  syncBootstrapSnapshotIfNeeded,
  syncBrokerData,
  syncTriggerCycle,
  updateDevice,
} from "../shared/connect";

export type { PairingFlowPhase, ConfirmPairingWithBootstrapResult } from "../shared/connect";

export {
  getAiProviders,
  listAiModels,
  setDefaultAiProvider,
  updateAiProviderSettings,
} from "../shared/ai-providers";

export {
  addAiThreadTag,
  deleteAiThread,
  getAiThread,
  getAiThreadMessages,
  getAiThreadTags,
  listAiThreads,
  removeAiThreadTag,
  updateAiThread,
  updateToolResult,
} from "../shared/ai-threads";

export {
  dismissHealthIssue,
  executeHealthFix,
  getDismissedHealthIssues,
  getHealthConfig,
  getHealthStatus,
  restoreHealthIssue,
  runHealthChecks,
  updateHealthConfig,
} from "../shared/health";

export { streamAiChat } from "../web/ai-streaming";

export {
  listenBrokerSyncComplete,
  listenBrokerSyncError,
  listenBrokerSyncStart,
  listenDatabaseRestored,
  listenDeepLink,
  listenFileDrop,
  listenFileDropCancelled,
  listenFileDropHover,
  listenMarketSyncComplete,
  listenMarketSyncError,
  listenMarketSyncStart,
  listenNavigateToRoute,
  listenPortfolioUpdateComplete,
  listenPortfolioUpdateError,
  listenPortfolioUpdateStart,
} from "../web/events";

export {
  openCsvFileDialog,
  openDatabaseFileDialog,
  openFileSaveDialog,
  openFolderDialog,
  openUrlInBrowser,
} from "../web/files";

export {
  backupDatabase,
  backupDatabaseToPath,
  checkForUpdates,
  getAppInfo,
  getPlatform,
  getSettings,
  installUpdate,
  isAutoUpdateCheckEnabled,
  restoreDatabase,
  updateSettings,
} from "../web/settings";

export {
  checkAddonUpdate,
  checkAllAddonUpdates,
  clearAddonStaging,
  downloadAddonForReview,
  extractAddon,
  extractAddonZip,
  fetchAddonStoreListings,
  getAddonRatings,
  getEnabledAddons,
  getEnabledAddonsOnStartup,
  getInstalledAddons,
  installAddon,
  installAddonFile,
  installAddonZip,
  installFromStaging,
  listInstalledAddons,
  loadAddon,
  loadAddonForRuntime,
  submitAddonRating,
  toggleAddon,
  uninstallAddon,
  updateAddon,
} from "../web/addons";

export {
  getFireSettings,
  saveFireSettings,
  calculateFireProjection,
  runFireMonteCarlo,
  runFireScenarioAnalysis,
  runFireSorr,
  runFireSensitivity,
  runFireStrategyComparison,
} from "../web/fire-planner";

export {
  syncComputeSas,
  syncComputeSharedSecret,
  syncDecrypt,
  syncDeriveDek,
  syncDeriveSessionKey,
  syncEncrypt,
  syncGenerateDeviceId,
  syncGenerateKeypair,
  syncGeneratePairingCode,
  syncGenerateRootKey,
  syncHashPairingCode,
  syncHmacSha256,
} from "../web/crypto";
