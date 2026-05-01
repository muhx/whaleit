// Web adapter core - Internal invoke function, COMMANDS map, and helpers
// This module exports invoke, logger, and platform constants for shared modules

import { notifyUnauthorized } from "@/lib/auth-token";
import type { Logger } from "../types";

import * as accountHandlers from "./modules/accounts";
import * as activityHandlers from "./modules/activities";
import * as transactionHandlers from "./modules/transactions";
import * as holdingHandlers from "./modules/holdings";
import * as portfolioHandlers from "./modules/portfolio";
import * as goalHandlers from "./modules/goals";
import * as exchangeRateHandlers from "./modules/exchange-rates";
import * as aiHandlers from "./modules/ai";
import * as connectHandlers from "./modules/connect";
import * as marketDataHandlers from "./modules/market-data";
import * as taxonomyHandlers from "./modules/taxonomies";
import * as healthHandlers from "./modules/health";
import * as deviceSyncHandlers from "./modules/device-sync";
import * as settingsHandlers from "./modules/settings";
import * as secretHandlers from "./modules/secrets";
import * as assetHandlers from "./modules/assets";
import * as addonHandlers from "./modules/addons";
import * as utilityHandlers from "./modules/utilities";
import * as alternativeAssetHandlers from "./modules/alternative-assets";

/** True when running in the desktop (Tauri) environment */
export const isDesktop = false;

/** True when running in the web environment */
export const isWeb = true;

export const API_PREFIX = "/api/v1";
export const EVENTS_ENDPOINT = `${API_PREFIX}/events/stream`;
export const AI_CHAT_STREAM_ENDPOINT = `${API_PREFIX}/ai/chat/stream`;

type CommandMap = Record<string, { method: string; path: string }>;

export const COMMANDS: CommandMap = {
  get_accounts: { method: "GET", path: "/accounts" },
  create_account: { method: "POST", path: "/accounts" },
  update_account: { method: "PUT", path: "/accounts" },
  delete_account: { method: "DELETE", path: "/accounts" },
  get_settings: { method: "GET", path: "/settings" },
  update_settings: { method: "PUT", path: "/settings" },
  is_auto_update_check_enabled: { method: "GET", path: "/settings/auto-update-enabled" },
  get_app_info: { method: "GET", path: "/app/info" },
  check_update: { method: "GET", path: "/app/check-update" },
  backup_database: { method: "POST", path: "/utilities/database/backup" },
  backup_database_to_path: { method: "POST", path: "/utilities/database/backup-to-path" },
  restore_database: { method: "POST", path: "/utilities/database/restore" },
  get_holdings: { method: "GET", path: "/holdings" },
  get_holding: { method: "GET", path: "/holdings/item" },
  get_asset_holdings: { method: "GET", path: "/holdings/by-asset" },
  get_historical_valuations: { method: "GET", path: "/valuations/history" },
  get_latest_valuations: { method: "GET", path: "/valuations/latest" },
  get_portfolio_allocations: { method: "GET", path: "/allocations" },
  // Snapshot management
  get_snapshots: { method: "GET", path: "/snapshots" },
  get_snapshot_by_date: { method: "GET", path: "/snapshots/holdings" },
  delete_snapshot: { method: "DELETE", path: "/snapshots" },
  save_manual_holdings: { method: "POST", path: "/snapshots" },
  import_holdings_csv: { method: "POST", path: "/snapshots/import" },
  check_holdings_import: { method: "POST", path: "/snapshots/import/check" },
  update_portfolio: { method: "POST", path: "/portfolio/update" },
  recalculate_portfolio: { method: "POST", path: "/portfolio/recalculate" },
  // Performance
  calculate_accounts_simple_performance: { method: "POST", path: "/performance/accounts/simple" },
  calculate_performance_history: { method: "POST", path: "/performance/history" },
  calculate_performance_summary: { method: "POST", path: "/performance/summary" },
  get_income_summary: { method: "GET", path: "/income/summary" },
  // Goals
  get_goals: { method: "GET", path: "/goals" },
  create_goal: { method: "POST", path: "/goals" },
  update_goal: { method: "PUT", path: "/goals" },
  delete_goal: { method: "DELETE", path: "/goals" },
  update_goal_allocations: { method: "POST", path: "/goals/allocations" },
  load_goals_allocations: { method: "GET", path: "/goals/allocations" },
  // FX
  get_latest_exchange_rates: { method: "GET", path: "/exchange-rates/latest" },
  update_exchange_rate: { method: "PUT", path: "/exchange-rates" },
  add_exchange_rate: { method: "POST", path: "/exchange-rates" },
  delete_exchange_rate: { method: "DELETE", path: "/exchange-rates" },
  // Activities
  search_activities: { method: "POST", path: "/activities/search" },
  create_activity: { method: "POST", path: "/activities" },
  update_activity: { method: "PUT", path: "/activities" },
  save_activities: { method: "POST", path: "/activities/bulk" },
  delete_activity: { method: "DELETE", path: "/activities" },
  // Activity import
  check_activities_import: { method: "POST", path: "/activities/import/check" },
  preview_import_assets: { method: "POST", path: "/activities/import/assets/preview" },
  import_activities: { method: "POST", path: "/activities/import" },
  get_account_import_mapping: { method: "GET", path: "/activities/import/mapping" },
  save_account_import_mapping: { method: "POST", path: "/activities/import/mapping" },
  link_account_template: { method: "POST", path: "/activities/import/templates/link" },
  list_import_templates: { method: "GET", path: "/activities/import/templates" },
  get_import_template: { method: "GET", path: "/activities/import/templates/item" },
  save_import_template: { method: "POST", path: "/activities/import/templates" },
  delete_import_template: { method: "DELETE", path: "/activities/import/templates" },
  // Transactions (Phase 4)
  search_transactions: { method: "POST", path: "/transactions/search" },
  get_transaction: { method: "GET", path: "/transactions/item" },
  create_transaction: { method: "POST", path: "/transactions" },
  update_transaction: { method: "PUT", path: "/transactions" },
  delete_transaction: { method: "DELETE", path: "/transactions" },
  list_running_balance: { method: "POST", path: "/transactions/running-balance" },
  get_account_recent_transactions: {
    method: "GET",
    path: "/transactions/by-account/recent",
  },
  preview_transaction_import: { method: "POST", path: "/transactions/import/preview" },
  detect_transaction_duplicates: {
    method: "POST",
    path: "/transactions/import/duplicates",
  },
  list_transaction_templates: { method: "GET", path: "/transactions/import/templates" },
  save_transaction_template: { method: "POST", path: "/transactions/import/templates" },
  delete_transaction_template: {
    method: "DELETE",
    path: "/transactions/import/templates",
  },
  get_transaction_template: {
    method: "GET",
    path: "/transactions/import/templates/item",
  },
  create_transfer: { method: "POST", path: "/transactions/transfer" },
  update_transfer_leg: { method: "PUT", path: "/transactions/transfer/leg" },
  break_transfer_pair: { method: "POST", path: "/transactions/transfer/break" },
  lookup_payee_category: {
    method: "POST",
    path: "/transactions/payee-category-memory/lookup",
  },
  list_payee_category_memory: {
    method: "GET",
    path: "/transactions/payee-category-memory",
  },
  // Market data providers
  get_exchanges: { method: "GET", path: "/exchanges" },
  get_market_data_providers: { method: "GET", path: "/providers" },
  get_market_data_providers_settings: { method: "GET", path: "/providers/settings" },
  update_market_data_provider_settings: { method: "PUT", path: "/providers/settings" },
  // Custom providers
  get_custom_providers: { method: "GET", path: "/custom-providers" },
  create_custom_provider: { method: "POST", path: "/custom-providers" },
  update_custom_provider: { method: "PUT", path: "/custom-providers" },
  delete_custom_provider: { method: "DELETE", path: "/custom-providers" },
  test_custom_provider_source: { method: "POST", path: "/custom-providers/test-source" },
  // Contribution limits
  get_contribution_limits: { method: "GET", path: "/limits" },
  create_contribution_limit: { method: "POST", path: "/limits" },
  update_contribution_limit: { method: "PUT", path: "/limits" },
  delete_contribution_limit: { method: "DELETE", path: "/limits" },
  calculate_deposits_for_contribution_limit: { method: "GET", path: "/limits" },
  // Asset profile
  get_assets: { method: "GET", path: "/assets" },
  create_asset: { method: "POST", path: "/assets" },
  delete_asset: { method: "DELETE", path: "/assets" },
  get_asset_profile: { method: "GET", path: "/assets/profile" },
  update_asset_profile: { method: "PUT", path: "/assets/profile" },
  update_quote_mode: { method: "PUT", path: "/assets/pricing-mode" },
  // Market data
  search_symbol: { method: "GET", path: "/market-data/search" },
  resolve_symbol_quote: { method: "GET", path: "/market-data/resolve-currency" },
  get_quote_history: { method: "GET", path: "/market-data/quotes/history" },
  get_latest_quotes: { method: "POST", path: "/market-data/quotes/latest" },
  update_quote: { method: "PUT", path: "/market-data/quotes" },
  delete_quote: { method: "DELETE", path: "/market-data/quotes/id" },
  check_quotes_import: { method: "POST", path: "/market-data/quotes/check" },
  import_quotes_csv: { method: "POST", path: "/market-data/quotes/import" },
  synch_quotes: { method: "POST", path: "/market-data/sync/history" },
  sync_market_data: { method: "POST", path: "/market-data/sync" },
  // Secrets
  set_secret: { method: "POST", path: "/secrets" },
  get_secret: { method: "GET", path: "/secrets" },
  delete_secret: { method: "DELETE", path: "/secrets" },
  // Taxonomies
  get_taxonomies: { method: "GET", path: "/taxonomies" },
  get_taxonomy: { method: "GET", path: "/taxonomies" },
  create_taxonomy: { method: "POST", path: "/taxonomies" },
  update_taxonomy: { method: "PUT", path: "/taxonomies" },
  delete_taxonomy: { method: "DELETE", path: "/taxonomies" },
  create_category: { method: "POST", path: "/taxonomies/categories" },
  update_category: { method: "PUT", path: "/taxonomies/categories" },
  delete_category: { method: "DELETE", path: "/taxonomies" },
  move_category: { method: "POST", path: "/taxonomies/categories/move" },
  import_taxonomy_json: { method: "POST", path: "/taxonomies/import" },
  export_taxonomy_json: { method: "GET", path: "/taxonomies" },
  get_asset_taxonomy_assignments: { method: "GET", path: "/taxonomies/assignments/asset" },
  assign_asset_to_category: { method: "POST", path: "/taxonomies/assignments" },
  remove_asset_taxonomy_assignment: { method: "DELETE", path: "/taxonomies/assignments" },
  get_migration_status: { method: "GET", path: "/taxonomies/migration/status" },
  migrate_legacy_classifications: { method: "POST", path: "/taxonomies/migration/run" },
  // Health Center
  get_health_status: { method: "GET", path: "/health/status" },
  run_health_checks: { method: "POST", path: "/health/check" },
  dismiss_health_issue: { method: "POST", path: "/health/dismiss" },
  restore_health_issue: { method: "POST", path: "/health/restore" },
  get_dismissed_health_issues: { method: "GET", path: "/health/dismissed" },
  execute_health_fix: { method: "POST", path: "/health/fix" },
  get_health_config: { method: "GET", path: "/health/config" },
  update_health_config: { method: "PUT", path: "/health/config" },
  // Addons
  list_installed_addons: { method: "GET", path: "/addons/installed" },
  install_addon_zip: { method: "POST", path: "/addons/install-zip" },
  toggle_addon: { method: "POST", path: "/addons/toggle" },
  uninstall_addon: { method: "DELETE", path: "/addons" },
  load_addon_for_runtime: { method: "GET", path: "/addons/runtime" },
  get_enabled_addons_on_startup: { method: "GET", path: "/addons/enabled-on-startup" },
  extract_addon_zip: { method: "POST", path: "/addons/extract" },
  // Addon store + staging
  fetch_addon_store_listings: { method: "GET", path: "/addons/store/listings" },
  submit_addon_rating: { method: "POST", path: "/addons/store/ratings" },
  get_addon_ratings: { method: "GET", path: "/addons/store/ratings" },
  check_addon_update: { method: "POST", path: "/addons/store/check-update" },
  check_all_addon_updates: { method: "POST", path: "/addons/store/check-all" },
  update_addon_from_store_by_id: { method: "POST", path: "/addons/store/update" },
  download_addon_to_staging: { method: "POST", path: "/addons/store/staging/download" },
  install_addon_from_staging: { method: "POST", path: "/addons/store/install-from-staging" },
  clear_addon_staging: { method: "DELETE", path: "/addons/store/staging" },
  // Device Sync - Device management
  register_device: { method: "POST", path: "/sync/device/register" },
  get_device: { method: "GET", path: "/sync/device" },
  list_devices: { method: "GET", path: "/sync/devices" },
  update_device: { method: "PATCH", path: "/sync/device" },
  delete_device: { method: "DELETE", path: "/sync/device" },
  revoke_device: { method: "POST", path: "/sync/device" },
  // Device Sync - Team keys (E2EE)
  initialize_team_keys: { method: "POST", path: "/sync/keys/initialize" },
  commit_initialize_team_keys: { method: "POST", path: "/sync/keys/initialize/commit" },
  rotate_team_keys: { method: "POST", path: "/sync/keys/rotate" },
  commit_rotate_team_keys: { method: "POST", path: "/sync/keys/rotate/commit" },
  reset_team_sync: { method: "POST", path: "/sync/team/reset" },
  // Device Sync - Pairing (Issuer - Trusted Device)
  create_pairing: { method: "POST", path: "/sync/pairing" },
  get_pairing: { method: "GET", path: "/sync/pairing" },
  approve_pairing: { method: "POST", path: "/sync/pairing" },
  complete_pairing: { method: "POST", path: "/sync/pairing" },
  cancel_pairing: { method: "POST", path: "/sync/pairing" },
  // Device Sync - Pairing (Claimer - New Device)
  claim_pairing: { method: "POST", path: "/sync/pairing/claim" },
  get_pairing_messages: { method: "GET", path: "/sync/pairing" },
  confirm_pairing: { method: "POST", path: "/sync/pairing" },
  complete_pairing_with_transfer: {
    method: "POST",
    path: "/sync/pairing/complete-with-transfer",
  },
  confirm_pairing_with_bootstrap: {
    method: "POST",
    path: "/sync/pairing/confirm-with-bootstrap",
  },
  begin_pairing_confirm: { method: "POST", path: "/sync/pairing/flow/begin" },
  get_pairing_flow_state: { method: "POST", path: "/sync/pairing/flow/state" },
  approve_pairing_overwrite: { method: "POST", path: "/sync/pairing/flow/approve-overwrite" },
  cancel_pairing_flow: { method: "POST", path: "/sync/pairing/flow/cancel" },
  // WhaleIt Connect (Broker Sync)
  store_sync_session: { method: "POST", path: "/connect/session" },
  clear_sync_session: { method: "DELETE", path: "/connect/session" },
  get_sync_session_status: { method: "GET", path: "/connect/session/status" },
  restore_sync_session: { method: "GET", path: "/connect/session/restore" },
  list_broker_connections: { method: "GET", path: "/connect/connections" },
  list_broker_accounts: { method: "GET", path: "/connect/accounts" },
  sync_broker_data: { method: "POST", path: "/connect/sync" },
  broker_ingest_run: { method: "POST", path: "/connect/sync" },
  sync_broker_connections: { method: "POST", path: "/connect/sync/connections" },
  sync_broker_accounts: { method: "POST", path: "/connect/sync/accounts" },
  sync_broker_activities: { method: "POST", path: "/connect/sync/activities" },
  get_subscription_plans: { method: "GET", path: "/connect/plans" },
  get_subscription_plans_public: { method: "GET", path: "/connect/plans/public" },
  get_user_info: { method: "GET", path: "/connect/user" },
  // Local data queries (from local database)
  get_synced_accounts: { method: "GET", path: "/connect/synced-accounts" },
  get_platforms: { method: "GET", path: "/connect/platforms" },
  get_broker_sync_states: { method: "GET", path: "/connect/sync-states" },
  get_broker_ingest_states: { method: "GET", path: "/connect/sync-states" },
  get_import_runs: { method: "GET", path: "/connect/import-runs" },
  get_data_import_runs: { method: "GET", path: "/connect/import-runs" },
  get_broker_sync_profile: { method: "GET", path: "/connect/broker-sync-profile" },
  save_broker_sync_profile_rules: { method: "POST", path: "/connect/broker-sync-profile" },
  // Device Sync / Enrollment
  get_device_sync_state: { method: "GET", path: "/connect/device/sync-state" },
  enable_device_sync: { method: "POST", path: "/connect/device/enable" },
  clear_device_sync_data: { method: "DELETE", path: "/connect/device/sync-data" },
  reinitialize_device_sync: { method: "POST", path: "/connect/device/reinitialize" },
  device_sync_engine_status: { method: "GET", path: "/connect/device/engine-status" },
  device_sync_pairing_source_status: {
    method: "GET",
    path: "/connect/device/pairing-source-status",
  },
  device_sync_bootstrap_overwrite_check: {
    method: "GET",
    path: "/connect/device/bootstrap-overwrite-check",
  },
  device_sync_reconcile_ready_state: {
    method: "POST",
    path: "/connect/device/reconcile-ready-state",
  },
  device_sync_bootstrap_snapshot_if_needed: {
    method: "POST",
    path: "/connect/device/bootstrap-snapshot",
  },
  device_sync_trigger_cycle: { method: "POST", path: "/connect/device/trigger-cycle" },
  device_sync_start_background_engine: {
    method: "POST",
    path: "/connect/device/start-background",
  },
  device_sync_stop_background_engine: {
    method: "POST",
    path: "/connect/device/stop-background",
  },
  device_sync_generate_snapshot_now: {
    method: "POST",
    path: "/connect/device/generate-snapshot",
  },
  device_sync_cancel_snapshot_upload: {
    method: "POST",
    path: "/connect/device/cancel-snapshot",
  },
  // Net Worth
  get_net_worth: { method: "GET", path: "/net-worth" },
  get_net_worth_history: { method: "GET", path: "/net-worth/history" },
  // AI Providers
  get_ai_providers: { method: "GET", path: "/ai/providers" },
  update_ai_provider_settings: { method: "PUT", path: "/ai/providers/settings" },
  set_default_ai_provider: { method: "POST", path: "/ai/providers/default" },
  list_ai_models: { method: "GET", path: "/ai/providers" },
  // AI Threads
  list_ai_threads: { method: "GET", path: "/ai/threads" },
  get_ai_thread: { method: "GET", path: "/ai/threads" },
  get_ai_thread_messages: { method: "GET", path: "/ai/threads" },
  update_ai_thread: { method: "PUT", path: "/ai/threads" },
  delete_ai_thread: { method: "DELETE", path: "/ai/threads" },
  add_ai_thread_tag: { method: "POST", path: "/ai/threads" },
  remove_ai_thread_tag: { method: "DELETE", path: "/ai/threads" },
  get_ai_thread_tags: { method: "GET", path: "/ai/threads" },
  update_tool_result: { method: "PATCH", path: "/ai/tool-result" },
  // Alternative Assets
  create_alternative_asset: { method: "POST", path: "/alternative-assets" },
  update_alternative_asset_valuation: { method: "PUT", path: "/alternative-assets" },
  delete_alternative_asset: { method: "DELETE", path: "/alternative-assets" },
  link_liability: { method: "POST", path: "/alternative-assets" },
  unlink_liability: { method: "DELETE", path: "/alternative-assets" },
  update_alternative_asset_metadata: { method: "PUT", path: "/alternative-assets" },
  get_alternative_holdings: { method: "GET", path: "/alternative-holdings" },
};

/**
 * Logger implementation using console
 */
export const logger: Logger = {
  error: (...args: unknown[]) => console.error(...args),
  warn: (...args: unknown[]) => console.warn(...args),
  info: (...args: unknown[]) => console.info(...args),
  debug: (...args: unknown[]) => console.debug(...args),
  trace: (...args: unknown[]) => console.trace(...args),
};

/**
 * Convert Uint8Array or number[] to base64 string
 */
export function toBase64(data: Uint8Array | number[]): string {
  const bytes = Array.isArray(data) ? new Uint8Array(data) : data;
  // Fast base64 encoding without TextEncoder for binary
  let binary = "";
  const len = bytes.byteLength;
  for (let i = 0; i < len; i++) {
    binary += String.fromCharCode(bytes[i]);
  }
  // btoa expects binary string
  return btoa(binary);
}

/**
 * Convert base64 string to Uint8Array
 */
export function fromBase64(value: string): Uint8Array {
  const binary = atob(value);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes;
}

type HandleResult = { url: string; body: string | undefined };

/**
 * Invoke a command via REST API (internal - use typed adapter functions instead)
 */
export const invoke = async <T>(command: string, payload?: Record<string, unknown>): Promise<T> => {
  const config = COMMANDS[command];
  if (!config) throw new Error(`Unsupported command ${command}`);
  let url = `${API_PREFIX}${config.path}`;

  const result = handleCommand(command, url, payload);
  url = result.url;
  const body = result.body;

  const headers: HeadersInit = {};
  if (body !== undefined) {
    headers["Content-Type"] = "application/json";
  }
  if (command === "get_health_status" || command === "run_health_checks") {
    const payloadTimezone =
      typeof payload === "object" && payload !== null && "clientTimezone" in payload
        ? String((payload as { clientTimezone?: string }).clientTimezone ?? "").trim()
        : "";
    const clientTimezone = payloadTimezone || Intl.DateTimeFormat().resolvedOptions().timeZone;
    if (clientTimezone) {
      headers["X-Client-Timezone"] = clientTimezone;
    }
  }

  const res = await fetch(url, {
    method: config.method,
    headers,
    body,
    credentials: "same-origin",
    signal: AbortSignal.timeout(300_000),
  });

  // 401 = app auth failure (JWT expired/invalid). Cloud auth failures return 403.
  if (res.status === 401) {
    notifyUnauthorized();
  }
  if (!res.ok) {
    let msg = res.statusText;
    try {
      const err = await res.json();
      msg = (err?.message ?? msg) as string;
    } catch (_e) {
      // ignore JSON parse error from non-JSON error bodies
      void 0;
    }
    console.error(`[Invoke] Command "${command}" failed: ${msg}`);
    throw new Error(msg);
  }
  if (command === "backup_database") {
    const parsed = (await res.json()) as { filename: string; dataB64: string };
    return {
      filename: parsed.filename,
      data: fromBase64(parsed.dataB64),
    } as T;
  }
  if (command === "backup_database_to_path") {
    const parsed = (await res.json()) as { path: string };
    return parsed.path as T;
  }
  // Handle responses with no body (204 No Content, 202 Accepted, or empty 200)
  if (res.status === 204 || res.status === 202) {
    return undefined as T;
  }
  const text = await res.text();
  if (!text) {
    return undefined as T;
  }
  return JSON.parse(text) as T;
};

/**
 * Dispatch command to the appropriate domain module handler.
 * Returns modified url and body for the HTTP request.
 */
export function handleCommand(
  command: string,
  url: string,
  payload?: Record<string, unknown>,
): HandleResult {
  const p = payload as Record<string, unknown> | undefined;

  switch (command) {
    // ── Accounts ────────────────────────────────────────────
    case "get_accounts":
      return accountHandlers.handleGetAccounts(url, p);
    case "create_account":
      return accountHandlers.handleCreateAccount(url, p!);
    case "update_account":
      return accountHandlers.handleUpdateAccount(url, p!);
    case "delete_account":
      return accountHandlers.handleDeleteAccount(url, p!);

    // ── Activities ──────────────────────────────────────────
    case "search_activities":
      return activityHandlers.handleSearchActivities(url, p!);
    case "create_activity":
      return activityHandlers.handleCreateActivity(url, p!);
    case "update_activity":
      return activityHandlers.handleUpdateActivity(url, p!);
    case "save_activities":
      return activityHandlers.handleSaveActivities(url, p!);
    case "delete_activity":
      return activityHandlers.handleDeleteActivity(url, p!);
    case "check_activities_import":
      return activityHandlers.handleCheckActivitiesImport(url, p!);
    case "preview_import_assets":
      return activityHandlers.handlePreviewImportAssets(url, p!);
    case "import_activities":
      return activityHandlers.handleImportActivities(url, p!);
    case "get_account_import_mapping":
      return activityHandlers.handleGetAccountImportMapping(url, p!);
    case "save_account_import_mapping":
      return activityHandlers.handleSaveAccountImportMapping(url, p!);
    case "get_import_template":
      return activityHandlers.handleGetImportTemplate(url, p!);
    case "delete_import_template":
      return activityHandlers.handleDeleteImportTemplate(url, p!);
    case "save_import_template":
      return activityHandlers.handleSaveImportTemplate(url, p!);
    case "link_account_template":
      return activityHandlers.handleLinkAccountTemplate(url, p!);

    // ── Transactions (Phase 4) ──────────────────────────────
    case "search_transactions":
      return transactionHandlers.handleSearchTransactions(url, p!);
    case "get_transaction":
      return transactionHandlers.handleGetTransaction(url, p!);
    case "create_transaction":
      return transactionHandlers.handleCreateTransaction(url, p!);
    case "update_transaction":
      return transactionHandlers.handleUpdateTransaction(url, p!);
    case "delete_transaction":
      return transactionHandlers.handleDeleteTransaction(url, p!);
    case "list_running_balance":
      return transactionHandlers.handleListRunningBalance(url, p!);
    case "get_account_recent_transactions":
      return transactionHandlers.handleGetAccountRecentTransactions(url, p!);
    case "preview_transaction_import":
      return transactionHandlers.handlePreviewTransactionImport(url, p!);
    case "detect_transaction_duplicates":
      return transactionHandlers.handleDetectTransactionDuplicates(url, p!);
    case "list_transaction_templates":
      return { url, body: undefined };
    case "save_transaction_template":
      return transactionHandlers.handleSaveTransactionTemplate(url, p!);
    case "delete_transaction_template":
      return transactionHandlers.handleDeleteTransactionTemplate(url, p!);
    case "get_transaction_template":
      return transactionHandlers.handleGetTransactionTemplate(url, p!);
    case "create_transfer":
      return transactionHandlers.handleCreateTransfer(url, p!);
    case "update_transfer_leg":
      return transactionHandlers.handleUpdateTransferLeg(url, p!);
    case "break_transfer_pair":
      return transactionHandlers.handleBreakTransferPair(url, p!);
    case "lookup_payee_category":
      return transactionHandlers.handleLookupPayeeCategory(url, p!);
    case "list_payee_category_memory":
      return transactionHandlers.handleListPayeeCategoryMemory(url, p!);

    // ── Holdings / Snapshots ────────────────────────────────
    case "get_holdings":
      return holdingHandlers.handleGetHoldings(url, p!);
    case "get_holding":
      return holdingHandlers.handleGetHolding(url, p!);
    case "get_asset_holdings":
      return holdingHandlers.handleGetAssetHoldings(url, p!);
    case "get_snapshots":
      return holdingHandlers.handleGetSnapshots(url, p!);
    case "get_snapshot_by_date":
      return holdingHandlers.handleGetSnapshotByDate(url, p!);
    case "delete_snapshot":
      return holdingHandlers.handleDeleteSnapshot(url, p!);
    case "save_manual_holdings":
      return holdingHandlers.handleSaveManualHoldings(url, p!);
    case "import_holdings_csv":
      return holdingHandlers.handleImportHoldingsCsv(url, p!);
    case "check_holdings_import":
      return holdingHandlers.handleCheckHoldingsImport(url, p!);

    // ── Portfolio / Performance / Valuations / Net Worth ────
    case "get_historical_valuations":
      return portfolioHandlers.handleGetHistoricalValuations(url, p!);
    case "get_latest_valuations":
      return portfolioHandlers.handleGetLatestValuations(url, p!);
    case "get_portfolio_allocations":
      return portfolioHandlers.handleGetPortfolioAllocations(url, p!);
    case "calculate_accounts_simple_performance":
      return portfolioHandlers.handleCalculateAccountsSimplePerformance(url, p);
    case "calculate_performance_history":
      return portfolioHandlers.handleCalculatePerformanceHistory(url, p!);
    case "calculate_performance_summary":
      return portfolioHandlers.handleCalculatePerformanceSummary(url, p!);
    case "get_income_summary":
      return portfolioHandlers.handleGetIncomeSummary(url, p!);
    case "get_net_worth":
      return portfolioHandlers.handleGetNetWorth(url, p!);
    case "get_net_worth_history":
      return portfolioHandlers.handleGetNetWorthHistory(url, p!);

    // ── Goals ───────────────────────────────────────────────
    case "create_goal":
      return goalHandlers.handleCreateGoal(url, p!);
    case "update_goal":
      return goalHandlers.handleUpdateGoal(url, p!);
    case "delete_goal":
      return goalHandlers.handleDeleteGoal(url, p!);
    case "update_goal_allocations":
      return goalHandlers.handleUpdateGoalAllocations(url, p!);

    // ── Exchange Rates ──────────────────────────────────────
    case "update_exchange_rate":
      return exchangeRateHandlers.handleUpdateExchangeRate(url, p!);
    case "add_exchange_rate":
      return exchangeRateHandlers.handleAddExchangeRate(url, p!);
    case "delete_exchange_rate":
      return exchangeRateHandlers.handleDeleteExchangeRate(url, p!);

    // ── AI ──────────────────────────────────────────────────
    case "get_ai_providers":
      return { url, body: undefined };
    case "update_ai_provider_settings":
      return aiHandlers.handleUpdateAiProviderSettings(url, p!);
    case "set_default_ai_provider":
      return aiHandlers.handleSetDefaultAiProvider(url, p!);
    case "list_ai_models":
      return aiHandlers.handleListAiModels(url, p!);
    case "list_ai_threads":
      return aiHandlers.handleListAiThreads(url, p);
    case "get_ai_thread":
      return aiHandlers.handleGetAiThread(url, p!);
    case "get_ai_thread_messages":
      return aiHandlers.handleGetAiThreadMessages(url, p!);
    case "update_tool_result":
      return aiHandlers.handleUpdateToolResult(url, p!);
    case "update_ai_thread":
      return aiHandlers.handleUpdateAiThread(url, p!);
    case "delete_ai_thread":
      return aiHandlers.handleDeleteAiThread(url, p!);
    case "add_ai_thread_tag":
      return aiHandlers.handleAddAiThreadTag(url, p!);
    case "remove_ai_thread_tag":
      return aiHandlers.handleRemoveAiThreadTag(url, p!);
    case "get_ai_thread_tags":
      return aiHandlers.handleGetAiThreadTags(url, p!);

    // ── Connect / Broker Sync ───────────────────────────────
    case "store_sync_session":
      return connectHandlers.handleStoreSyncSession(url, p!);
    case "get_import_runs":
    case "get_data_import_runs":
      return connectHandlers.handleGetImportRuns(url, p);
    case "get_broker_sync_profile":
      return connectHandlers.handleGetBrokerSyncProfile(url, p!);
    case "save_broker_sync_profile_rules":
      return connectHandlers.handleSaveBrokerSyncProfileRules(url, p!);
    case "device_sync_reconcile_ready_state":
      return connectHandlers.handleDeviceSyncReconcileReadyState(url, p!);
    // Connect commands with no payload transformation
    case "clear_sync_session":
    case "get_sync_session_status":
    case "restore_sync_session":
    case "list_broker_connections":
    case "list_broker_accounts":
    case "sync_broker_data":
    case "broker_ingest_run":
    case "sync_broker_connections":
    case "sync_broker_accounts":
    case "sync_broker_activities":
    case "get_subscription_plans":
    case "get_subscription_plans_public":
    case "get_user_info":
    case "get_synced_accounts":
    case "get_platforms":
    case "get_broker_sync_states":
    case "get_broker_ingest_states":
    case "get_device_sync_state": // falls through - Device Sync / Enrollment (no payload transformation)
    case "enable_device_sync":
    case "clear_device_sync_data":
    case "reinitialize_device_sync":
    case "device_sync_engine_status":
    case "device_sync_pairing_source_status":
    case "device_sync_bootstrap_overwrite_check":
    case "device_sync_bootstrap_snapshot_if_needed":
    case "device_sync_trigger_cycle":
    case "device_sync_start_background_engine":
    case "device_sync_stop_background_engine":
    case "device_sync_generate_snapshot_now":
    case "device_sync_cancel_snapshot_upload":
      return { url, body: undefined };

    // ── Market Data ─────────────────────────────────────────
    case "search_symbol":
      return marketDataHandlers.handleSearchSymbol(url, p!);
    case "resolve_symbol_quote":
      return marketDataHandlers.handleResolveSymbolQuote(url, p!);
    case "get_quote_history":
      return marketDataHandlers.handleGetQuoteHistory(url, p!);
    case "get_latest_quotes":
      return marketDataHandlers.handleGetLatestQuotes(url, p!);
    case "update_quote":
      return marketDataHandlers.handleUpdateQuote(url, p!);
    case "delete_quote":
      return marketDataHandlers.handleDeleteQuote(url, p!);
    case "check_quotes_import":
      return marketDataHandlers.handleCheckQuotesImport(url, p!);
    case "import_quotes_csv":
      return marketDataHandlers.handleImportQuotesCsv(url, p!);
    case "sync_market_data":
      return marketDataHandlers.handleSyncMarketData(url, p!);
    case "update_market_data_provider_settings":
      return marketDataHandlers.handleUpdateMarketDataProviderSettings(url, p!);
    case "get_exchanges":
    case "synch_quotes":
      return { url, body: undefined };
    // Custom providers
    case "create_custom_provider":
      return marketDataHandlers.handleCreateCustomProvider(url, p!);
    case "update_custom_provider":
      return marketDataHandlers.handleUpdateCustomProvider(url, p!);
    case "delete_custom_provider":
      return marketDataHandlers.handleDeleteCustomProvider(url, p!);
    case "test_custom_provider_source":
      return marketDataHandlers.handleTestCustomProviderSource(url, p!);

    // ── Taxonomies ──────────────────────────────────────────
    case "get_taxonomies":
    case "get_migration_status":
    case "migrate_legacy_classifications":
      return { url, body: undefined };
    case "get_taxonomy":
      return taxonomyHandlers.handleGetTaxonomy(url, p!);
    case "create_taxonomy":
      return taxonomyHandlers.handleCreateTaxonomy(url, p!);
    case "update_taxonomy":
      return taxonomyHandlers.handleUpdateTaxonomy(url, p!);
    case "delete_taxonomy":
      return taxonomyHandlers.handleDeleteTaxonomy(url, p!);
    case "create_category":
      return taxonomyHandlers.handleCreateCategory(url, p!);
    case "update_category":
      return taxonomyHandlers.handleUpdateCategory(url, p!);
    case "delete_category":
      return taxonomyHandlers.handleDeleteCategory(url, p!);
    case "move_category":
      return taxonomyHandlers.handleMoveCategory(url, p!);
    case "import_taxonomy_json":
      return taxonomyHandlers.handleImportTaxonomyJson(url, p!);
    case "export_taxonomy_json":
      return taxonomyHandlers.handleExportTaxonomyJson(url, p!);
    case "get_asset_taxonomy_assignments":
      return taxonomyHandlers.handleGetAssetTaxonomyAssignments(url, p!);
    case "assign_asset_to_category":
      return taxonomyHandlers.handleAssignAssetToCategory(url, p!);
    case "remove_asset_taxonomy_assignment":
      return taxonomyHandlers.handleRemoveAssetTaxonomyAssignment(url, p!);

    // ── Health Center ───────────────────────────────────────
    case "get_health_status":
    case "run_health_checks":
    case "get_dismissed_health_issues":
    case "get_health_config":
      return { url, body: undefined };
    case "dismiss_health_issue":
      return healthHandlers.handleDismissHealthIssue(url, p!);
    case "restore_health_issue":
      return healthHandlers.handleRestoreHealthIssue(url, p!);
    case "execute_health_fix":
      return healthHandlers.handleExecuteHealthFix(url, p!);
    case "update_health_config":
      return healthHandlers.handleUpdateHealthConfig(url, p!);

    // ── Addons ──────────────────────────────────────────────
    case "install_addon_zip":
      return addonHandlers.handleInstallAddonZip(url, p!);
    case "toggle_addon":
      return addonHandlers.handleToggleAddon(url, p!);
    case "uninstall_addon":
      return addonHandlers.handleUninstallAddon(url, p!);
    case "load_addon_for_runtime":
      return addonHandlers.handleLoadAddonForRuntime(url, p!);
    case "extract_addon_zip":
      return addonHandlers.handleExtractAddonZip(url, p!);
    case "check_addon_update":
      return addonHandlers.handleCheckAddonUpdate(url, p!);
    case "update_addon_from_store_by_id":
      return addonHandlers.handleUpdateAddonFromStoreById(url, p!);
    case "download_addon_to_staging":
      return addonHandlers.handleDownloadAddonToStaging(url, p!);
    case "install_addon_from_staging":
      return addonHandlers.handleInstallAddonFromStaging(url, p!);
    case "clear_addon_staging":
      return addonHandlers.handleClearAddonStaging(url, p);
    case "submit_addon_rating":
      return addonHandlers.handleSubmitAddonRating(url, p!);
    case "get_addon_ratings":
      return addonHandlers.handleGetAddonRatings(url, p!);
    case "check_all_addon_updates":
      return { url, body: undefined };

    // ── Device Sync ─────────────────────────────────────────
    case "register_device":
      return deviceSyncHandlers.handleRegisterDevice(url, p!);
    case "get_device":
      return deviceSyncHandlers.handleGetDevice(url, p);
    case "update_device":
      return deviceSyncHandlers.handleUpdateDevice(url, p!);
    case "delete_device":
      return deviceSyncHandlers.handleDeleteDevice(url, p!);
    case "revoke_device":
      return deviceSyncHandlers.handleRevokeDevice(url, p!);
    case "commit_initialize_team_keys":
      return deviceSyncHandlers.handleCommitInitializeTeamKeys(url, p!);
    case "commit_rotate_team_keys":
      return deviceSyncHandlers.handleCommitRotateTeamKeys(url, p!);
    case "reset_team_sync":
      return deviceSyncHandlers.handleResetTeamSync(url, p!);
    case "create_pairing":
      return deviceSyncHandlers.handleCreatePairing(url, p!);
    case "get_pairing":
      return deviceSyncHandlers.handleGetPairing(url, p!);
    case "approve_pairing":
      return deviceSyncHandlers.handleApprovePairing(url, p!);
    case "complete_pairing":
      return deviceSyncHandlers.handleCompletePairing(url, p!);
    case "cancel_pairing":
      return deviceSyncHandlers.handleCancelPairing(url, p!);
    case "claim_pairing":
      return deviceSyncHandlers.handleClaimPairing(url, p!);
    case "get_pairing_messages":
      return deviceSyncHandlers.handleGetPairingMessages(url, p!);
    case "confirm_pairing":
      return deviceSyncHandlers.handleConfirmPairing(url, p!);
    case "complete_pairing_with_transfer":
      return deviceSyncHandlers.handleCompletePairingWithTransfer(url, p!);
    case "confirm_pairing_with_bootstrap":
      return deviceSyncHandlers.handleConfirmPairingWithBootstrap(url, p!);
    case "begin_pairing_confirm":
      return deviceSyncHandlers.handleBeginPairingConfirm(url, p!);
    case "get_pairing_flow_state":
      return deviceSyncHandlers.handleGetPairingFlowState(url, p!);
    case "approve_pairing_overwrite":
      return deviceSyncHandlers.handleApprovePairingOverwrite(url, p!);
    case "cancel_pairing_flow":
      return deviceSyncHandlers.handleCancelPairingFlow(url, p!);
    case "list_devices":
    case "initialize_team_keys":
    case "rotate_team_keys":
      return { url, body: undefined };

    // ── Settings ────────────────────────────────────────────
    case "update_settings":
      return settingsHandlers.handleUpdateSettings(url, p!);
    case "check_update":
      return settingsHandlers.handleCheckUpdate(url, p);

    // ── Secrets ─────────────────────────────────────────────
    case "set_secret":
      return secretHandlers.handleSetSecret(url, p!);
    case "get_secret":
      return secretHandlers.handleGetSecret(url, p!);
    case "delete_secret":
      return secretHandlers.handleDeleteSecret(url, p!);

    // ── Assets ──────────────────────────────────────────────
    case "create_asset":
      return assetHandlers.handleCreateAsset(url, p!);
    case "delete_asset":
      return assetHandlers.handleDeleteAsset(url, p!);
    case "get_asset_profile":
      return assetHandlers.handleGetAssetProfile(url, p!);
    case "update_asset_profile":
      return assetHandlers.handleUpdateAssetProfile(url, p!);
    case "update_quote_mode":
      return assetHandlers.handleUpdateQuoteMode(url, p!);
    // Contribution limits
    case "create_contribution_limit":
      return assetHandlers.handleCreateContributionLimit(url, p!);
    case "update_contribution_limit":
      return assetHandlers.handleUpdateContributionLimit(url, p!);
    case "delete_contribution_limit":
      return assetHandlers.handleDeleteContributionLimit(url, p!);
    case "calculate_deposits_for_contribution_limit":
      return assetHandlers.handleCalculateDepositsForContributionLimit(url, p!);

    // ── Utilities ───────────────────────────────────────────
    case "backup_database_to_path":
      return utilityHandlers.handleBackupDatabaseToPath(url, p!);
    case "restore_database":
      return utilityHandlers.handleRestoreDatabase(url, p!);

    // ── Alternative Assets ──────────────────────────────────
    case "create_alternative_asset":
      return alternativeAssetHandlers.handleCreateAlternativeAsset(url, p!);
    case "update_alternative_asset_valuation":
      return alternativeAssetHandlers.handleUpdateAlternativeAssetValuation(url, p!);
    case "delete_alternative_asset":
      return alternativeAssetHandlers.handleDeleteAlternativeAsset(url, p!);
    case "link_liability":
      return alternativeAssetHandlers.handleLinkLiability(url, p!);
    case "unlink_liability":
      return alternativeAssetHandlers.handleUnlinkLiability(url, p!);
    case "update_alternative_asset_metadata":
      return alternativeAssetHandlers.handleUpdateAlternativeAssetMetadata(url, p!);
    case "get_alternative_holdings":
      return { url, body: undefined };

    // ── Settings / App info (no payload transformation) ─────
    case "get_settings":
    case "is_auto_update_check_enabled":
    case "get_app_info":
    case "backup_database":
    case "update_portfolio":
    case "recalculate_portfolio":
    case "get_goals":
    case "load_goals_allocations":
    case "get_latest_exchange_rates":
    case "get_market_data_providers":
    case "get_market_data_providers_settings":
    case "get_custom_providers":
    case "get_contribution_limits":
    case "get_assets":
    case "list_import_templates":
    case "list_installed_addons":
    case "get_enabled_addons_on_startup":
    case "fetch_addon_store_listings":
      return { url, body: undefined };

    default:
      // Unknown command — should not reach here if COMMANDS map is exhaustive
      return { url, body: undefined };
  }
}
