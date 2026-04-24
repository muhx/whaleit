-- Drop all tables in reverse dependency order

DROP TABLE IF EXISTS sync_device_config;
DROP TABLE IF EXISTS sync_table_state;
DROP TABLE IF EXISTS sync_entity_metadata;
DROP TABLE IF EXISTS sync_applied_events;
DROP TABLE IF EXISTS sync_outbox;
DROP TABLE IF EXISTS sync_engine_state;
DROP TABLE IF EXISTS sync_cursor;

DROP TABLE IF EXISTS ai_thread_tags;
DROP TABLE IF EXISTS ai_messages;
DROP TABLE IF EXISTS ai_threads;

DROP TABLE IF EXISTS brokers_sync_state;
DROP TABLE IF EXISTS import_account_templates;
DROP TABLE IF EXISTS import_templates;
DROP TABLE IF EXISTS import_runs;
DROP TABLE IF EXISTS platforms;

DROP TABLE IF EXISTS market_data_custom_providers;
DROP TABLE IF EXISTS market_data_providers;
DROP TABLE IF EXISTS quote_sync_state;
DROP TABLE IF EXISTS quotes;

DROP TABLE IF EXISTS asset_taxonomy_assignments;
DROP TABLE IF EXISTS taxonomy_categories;
DROP TABLE IF EXISTS taxonomies;

DROP TABLE IF EXISTS contribution_limits;
DROP TABLE IF EXISTS health_issue_dismissals;
DROP TABLE IF EXISTS goals_allocation;
DROP TABLE IF EXISTS goals;

DROP TABLE IF EXISTS daily_account_valuation;
DROP TABLE IF EXISTS holdings_snapshots;

DROP TABLE IF EXISTS activities;
DROP TABLE IF EXISTS assets;
DROP TABLE IF EXISTS accounts;
DROP TABLE IF EXISTS app_settings;
