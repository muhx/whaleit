-- Initial PostgreSQL schema for WhaleIt
-- Consolidated migration creating all tables.
-- Mirrors SQLite schema with native PostgreSQL types:
--   - IDs: TEXT (UUID v7 stored as string for core compatibility)
--   - Booleans: BOOLEAN (native PG)
--   - Timestamps: TIMESTAMP (without timezone, maps to NaiveDateTime)
--   - Decimals: TEXT (serialized Rust Decimal)
--   - JSON: TEXT (serialized serde_json::Value)

-- Enable pgcrypto extension for gen_random_uuid() if needed later
-- CREATE EXTENSION IF NOT EXISTS pgcrypto;

----------------------------------------------------------------------
-- Core domain tables
----------------------------------------------------------------------

CREATE TABLE app_settings (
    setting_key TEXT PRIMARY KEY,
    setting_value TEXT NOT NULL
);

CREATE TABLE platforms (
    id TEXT PRIMARY KEY,
    name TEXT,
    url TEXT NOT NULL,
    external_id TEXT,
    kind TEXT NOT NULL DEFAULT 'GENERIC',
    website_url TEXT,
    logo_url TEXT
);

CREATE TABLE accounts (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    account_type TEXT NOT NULL,
    "group" TEXT,
    currency TEXT NOT NULL,
    is_default BOOLEAN NOT NULL DEFAULT FALSE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    platform_id TEXT REFERENCES platforms(id),
    account_number TEXT,
    meta TEXT,
    provider TEXT,
    provider_account_id TEXT,
    is_archived BOOLEAN NOT NULL DEFAULT FALSE,
    tracking_mode TEXT NOT NULL DEFAULT 'NOT_SET'
);

CREATE TABLE assets (
    id TEXT PRIMARY KEY,
    kind TEXT NOT NULL,
    name TEXT,
    display_code TEXT,
    notes TEXT,
    metadata TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    quote_mode TEXT NOT NULL DEFAULT 'MARKET',
    quote_ccy TEXT NOT NULL,
    instrument_type TEXT,
    instrument_symbol TEXT,
    instrument_exchange_mic TEXT,
    instrument_key TEXT,
    provider_config TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE import_runs (
    id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES accounts(id),
    source_system TEXT NOT NULL,
    run_type TEXT NOT NULL,
    mode TEXT NOT NULL DEFAULT 'REVIEW',
    status TEXT NOT NULL DEFAULT 'STARTED',
    started_at TIMESTAMP NOT NULL DEFAULT NOW(),
    finished_at TIMESTAMP,
    review_mode TEXT NOT NULL DEFAULT 'STANDARD',
    applied_at TIMESTAMP,
    checkpoint_in TEXT,
    checkpoint_out TEXT,
    summary TEXT,
    warnings TEXT,
    error TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE activities (
    id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES accounts(id),
    asset_id TEXT REFERENCES assets(id),
    activity_type TEXT NOT NULL,
    activity_type_override TEXT,
    source_type TEXT,
    subtype TEXT,
    status TEXT NOT NULL DEFAULT 'POSTED',
    activity_date TEXT NOT NULL,
    settlement_date TEXT,
    quantity TEXT,
    unit_price TEXT,
    amount TEXT,
    fee TEXT,
    currency TEXT NOT NULL,
    fx_rate TEXT,
    notes TEXT,
    metadata TEXT,
    source_system TEXT,
    source_record_id TEXT,
    source_group_id TEXT,
    idempotency_key TEXT,
    import_run_id TEXT REFERENCES import_runs(id),
    is_user_modified BOOLEAN NOT NULL DEFAULT FALSE,
    needs_review BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

----------------------------------------------------------------------
-- Portfolio tables
----------------------------------------------------------------------

CREATE TABLE holdings_snapshots (
    id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL,
    snapshot_date DATE NOT NULL,
    currency TEXT NOT NULL,
    positions TEXT NOT NULL DEFAULT '{}',
    cash_balances TEXT NOT NULL DEFAULT '{}',
    cost_basis TEXT NOT NULL DEFAULT '0',
    net_contribution TEXT NOT NULL DEFAULT '0',
    calculated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    net_contribution_base TEXT NOT NULL DEFAULT '0',
    cash_total_account_currency TEXT NOT NULL DEFAULT '0',
    cash_total_base_currency TEXT NOT NULL DEFAULT '0',
    source TEXT NOT NULL DEFAULT 'MANUAL',
    UNIQUE(account_id, snapshot_date)
);

CREATE TABLE daily_account_valuation (
    id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL,
    valuation_date DATE NOT NULL,
    account_currency TEXT NOT NULL,
    base_currency TEXT NOT NULL,
    fx_rate_to_base TEXT NOT NULL DEFAULT '1',
    cash_balance TEXT NOT NULL DEFAULT '0',
    investment_market_value TEXT NOT NULL DEFAULT '0',
    total_value TEXT NOT NULL DEFAULT '0',
    cost_basis TEXT NOT NULL DEFAULT '0',
    net_contribution TEXT NOT NULL DEFAULT '0',
    calculated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    UNIQUE(account_id, valuation_date)
);

----------------------------------------------------------------------
-- Goals tables
----------------------------------------------------------------------

CREATE TABLE goals (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT,
    target_amount DOUBLE PRECISION NOT NULL,
    is_achieved BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE goals_allocation (
    id TEXT PRIMARY KEY,
    percent_allocation INTEGER NOT NULL,
    goal_id TEXT NOT NULL REFERENCES goals(id),
    account_id TEXT NOT NULL REFERENCES accounts(id)
);

----------------------------------------------------------------------
-- Health tables
----------------------------------------------------------------------

CREATE TABLE health_issue_dismissals (
    issue_id TEXT PRIMARY KEY,
    dismissed_at TIMESTAMP NOT NULL DEFAULT NOW(),
    data_hash TEXT NOT NULL
);

----------------------------------------------------------------------
-- Limits tables
----------------------------------------------------------------------

CREATE TABLE contribution_limits (
    id TEXT PRIMARY KEY,
    group_name TEXT NOT NULL,
    contribution_year INTEGER NOT NULL,
    limit_amount DOUBLE PRECISION NOT NULL,
    account_ids TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    start_date TEXT,
    end_date TEXT
);

----------------------------------------------------------------------
-- Taxonomies tables
----------------------------------------------------------------------

CREATE TABLE taxonomies (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    color TEXT NOT NULL,
    description TEXT,
    is_system BOOLEAN NOT NULL DEFAULT FALSE,
    is_single_select BOOLEAN NOT NULL DEFAULT FALSE,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE taxonomy_categories (
    id TEXT NOT NULL,
    taxonomy_id TEXT NOT NULL REFERENCES taxonomies(id),
    parent_id TEXT,
    name TEXT NOT NULL,
    key TEXT NOT NULL,
    color TEXT NOT NULL,
    description TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    PRIMARY KEY (id, taxonomy_id)
);

CREATE TABLE asset_taxonomy_assignments (
    id TEXT PRIMARY KEY,
    asset_id TEXT NOT NULL REFERENCES assets(id),
    taxonomy_id TEXT NOT NULL REFERENCES taxonomies(id),
    category_id TEXT NOT NULL,
    weight INTEGER NOT NULL DEFAULT 10000,
    source TEXT NOT NULL DEFAULT 'USER',
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    UNIQUE(asset_id, taxonomy_id, category_id)
);

----------------------------------------------------------------------
-- Market data tables
----------------------------------------------------------------------

CREATE TABLE quotes (
    id TEXT PRIMARY KEY,
    asset_id TEXT NOT NULL REFERENCES assets(id),
    day TEXT NOT NULL,
    source TEXT NOT NULL,
    open TEXT,
    high TEXT,
    low TEXT,
    close TEXT NOT NULL,
    adjclose TEXT,
    volume TEXT,
    currency TEXT NOT NULL,
    notes TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    timestamp TIMESTAMP NOT NULL DEFAULT NOW(),
    UNIQUE(asset_id, day, source)
);

CREATE TABLE quote_sync_state (
    asset_id TEXT PRIMARY KEY REFERENCES assets(id),
    position_closed_date TEXT,
    last_synced_at TIMESTAMP,
    data_source TEXT NOT NULL DEFAULT 'YAHOO',
    sync_priority INTEGER NOT NULL DEFAULT 0,
    error_count INTEGER NOT NULL DEFAULT 0,
    last_error TEXT,
    profile_enriched_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE market_data_providers (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    url TEXT,
    priority INTEGER NOT NULL DEFAULT 0,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    logo_filename TEXT,
    last_synced_at TIMESTAMP,
    last_sync_status TEXT,
    last_sync_error TEXT,
    provider_type TEXT NOT NULL DEFAULT 'YAHOO',
    config TEXT
);

CREATE TABLE market_data_custom_providers (
    id TEXT PRIMARY KEY,
    code TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    priority INTEGER NOT NULL DEFAULT 0,
    config TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

----------------------------------------------------------------------
-- Import / sync tables
----------------------------------------------------------------------

CREATE TABLE import_templates (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    scope TEXT NOT NULL DEFAULT 'ACCOUNT',
    kind TEXT NOT NULL DEFAULT 'CSV',
    source_system TEXT NOT NULL DEFAULT 'MANUAL',
    config_version INTEGER NOT NULL DEFAULT 1,
    config TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE import_account_templates (
    id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES accounts(id),
    context_kind TEXT NOT NULL DEFAULT 'DEFAULT',
    source_system TEXT NOT NULL DEFAULT 'MANUAL',
    template_id TEXT NOT NULL REFERENCES import_templates(id),
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    UNIQUE(account_id, context_kind, source_system)
);

CREATE TABLE brokers_sync_state (
    account_id TEXT NOT NULL REFERENCES accounts(id),
    provider TEXT NOT NULL,
    checkpoint_json TEXT,
    last_attempted_at TIMESTAMP,
    last_successful_at TIMESTAMP,
    last_error TEXT,
    last_run_id TEXT REFERENCES import_runs(id),
    sync_status TEXT NOT NULL DEFAULT 'IDLE',
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    PRIMARY KEY (account_id, provider)
);

----------------------------------------------------------------------
-- AI chat tables
----------------------------------------------------------------------

CREATE TABLE ai_threads (
    id TEXT PRIMARY KEY,
    title TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    config_snapshot TEXT,
    is_pinned BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE ai_messages (
    id TEXT PRIMARY KEY,
    thread_id TEXT NOT NULL REFERENCES ai_threads(id),
    role TEXT NOT NULL,
    content_json TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE ai_thread_tags (
    id TEXT PRIMARY KEY,
    thread_id TEXT NOT NULL REFERENCES ai_threads(id),
    tag TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    UNIQUE(thread_id, tag)
);

----------------------------------------------------------------------
-- Device sync tables
----------------------------------------------------------------------

CREATE TABLE sync_cursor (
    id INTEGER PRIMARY KEY DEFAULT 1,
    cursor BIGINT NOT NULL DEFAULT 0,
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE sync_engine_state (
    id INTEGER PRIMARY KEY DEFAULT 1,
    lock_version BIGINT NOT NULL DEFAULT 0,
    last_push_at TIMESTAMP,
    last_pull_at TIMESTAMP,
    last_error TEXT,
    consecutive_failures INTEGER NOT NULL DEFAULT 0,
    next_retry_at TIMESTAMP,
    last_cycle_status TEXT,
    last_cycle_duration_ms BIGINT
);

CREATE TABLE sync_outbox (
    event_id TEXT PRIMARY KEY,
    entity TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    op TEXT NOT NULL,
    client_timestamp TIMESTAMP NOT NULL DEFAULT NOW(),
    payload TEXT NOT NULL,
    payload_key_version INTEGER NOT NULL DEFAULT 0,
    sent BOOLEAN NOT NULL DEFAULT FALSE,
    status TEXT NOT NULL DEFAULT 'PENDING',
    retry_count INTEGER NOT NULL DEFAULT 0,
    next_retry_at TIMESTAMP,
    last_error TEXT,
    last_error_code TEXT,
    device_id TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE sync_applied_events (
    event_id TEXT PRIMARY KEY,
    seq BIGINT NOT NULL,
    entity TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    applied_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE sync_entity_metadata (
    entity TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    last_event_id TEXT NOT NULL,
    last_client_timestamp TIMESTAMP NOT NULL DEFAULT NOW(),
    last_seq BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (entity, entity_id)
);

CREATE TABLE sync_table_state (
    table_name TEXT PRIMARY KEY,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    last_snapshot_restore_at TIMESTAMP,
    last_incremental_apply_at TIMESTAMP
);

CREATE TABLE sync_device_config (
    device_id TEXT PRIMARY KEY,
    key_version INTEGER,
    trust_state TEXT NOT NULL DEFAULT 'UNKNOWN',
    last_bootstrap_at TIMESTAMP,
    min_snapshot_created_at TIMESTAMP
);

----------------------------------------------------------------------
-- Indexes
----------------------------------------------------------------------

CREATE INDEX idx_activities_account_id ON activities(account_id);
CREATE INDEX idx_activities_asset_id ON activities(asset_id);
CREATE INDEX idx_activities_type ON activities(activity_type);
CREATE INDEX idx_activities_date ON activities(activity_date);
CREATE INDEX idx_activities_idempotency_key ON activities(idempotency_key);
CREATE INDEX idx_activities_import_run_id ON activities(import_run_id);

CREATE INDEX idx_assets_kind ON assets(kind);
CREATE INDEX idx_assets_instrument_key ON assets(instrument_key);
CREATE INDEX idx_assets_is_active ON assets(is_active);
CREATE INDEX idx_assets_instrument_symbol ON assets(instrument_symbol);

CREATE INDEX idx_holdings_snapshots_account_date ON holdings_snapshots(account_id, snapshot_date);
CREATE INDEX idx_daily_valuation_account_date ON daily_account_valuation(account_id, valuation_date);

CREATE INDEX idx_quotes_asset_day ON quotes(asset_id, day);
CREATE INDEX idx_quotes_timestamp ON quotes(timestamp);

CREATE INDEX idx_goals_allocation_goal ON goals_allocation(goal_id);
CREATE INDEX idx_goals_allocation_account ON goals_allocation(account_id);

CREATE INDEX idx_contribution_limits_year ON contribution_limits(contribution_year);

CREATE INDEX idx_asset_taxonomy_asset ON asset_taxonomy_assignments(asset_id);
CREATE INDEX idx_asset_taxonomy_taxonomy ON asset_taxonomy_assignments(taxonomy_id);

CREATE INDEX idx_import_runs_account ON import_runs(account_id);
CREATE INDEX idx_import_runs_status ON import_runs(status);

CREATE INDEX idx_brokers_sync_account ON brokers_sync_state(account_id);

CREATE INDEX idx_sync_outbox_entity ON sync_outbox(entity, entity_id);
CREATE INDEX idx_sync_outbox_status ON sync_outbox(status);
CREATE INDEX idx_sync_outbox_created ON sync_outbox(created_at);

CREATE INDEX idx_sync_applied_events_seq ON sync_applied_events(seq);
