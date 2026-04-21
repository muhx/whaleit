//! Parity tests comparing SQLite and PostgreSQL repository behavior.
//!
//! These tests verify that both database engines produce identical results
//! when given the same input data. Tests are marked with `#[ignore]` by default
//! and should be run with `cargo test -p whaleit-storage-postgres -- --ignored`
//! or via CI with PostgreSQL service container.

use std::sync::Arc;

use chrono::Utc;
use whaleit_core::accounts::{AccountRepositoryTrait, AccountUpdate, NewAccount, TrackingMode};
use whaleit_core::assets::{AssetKind, AssetRepositoryTrait, InstrumentType, NewAsset, QuoteMode};
use whaleit_core::fx::{ExchangeRate, FxRepositoryTrait, NewExchangeRate};
use whaleit_core::goals::{GoalRepositoryTrait, NewGoal};
use whaleit_core::health::{HealthDismissalStore, IssueDismissal};
use whaleit_core::limits::{ContributionLimitRepositoryTrait, NewContributionLimit};
use whaleit_core::portfolio::snapshot::{SnapshotRepositoryTrait, AccountStateSnapshot, SnapshotSource};
use whaleit_core::portfolio::valuation::{ValuationRepositoryTrait, DailyAccountValuation};
use whaleit_core::quotes::{QuoteStore, Quote};
use whaleit_core::settings::SettingsRepositoryTrait;
use whaleit_core::taxonomies::{NewTaxonomy, TaxonomyRepositoryTrait};

/// Creates an in-memory SQLite repository for testing.
fn create_sqlite_repo() -> SqliteRepos {
    use whaleit_storage_sqlite::accounts::AccountRepository;
    use whaleit_storage_sqlite::assets::AssetRepository;
    use whaleit_storage_sqlite::fx::FxRepository;
    use whaleit_storage_sqlite::goals::GoalRepository;
    use whaleit_storage_sqlite::health::HealthDismissalRepository;
    use whaleit_storage_sqlite::limits::ContributionLimitRepository;
    use whaleit_storage_sqlite::market_data::MarketDataRepository;
    use whaleit_storage_sqlite::portfolio::snapshot::SnapshotRepository;
    use whaleit_storage_sqlite::portfolio::valuation::ValuationRepository;
    use whaleit_storage_sqlite::settings::SettingsRepository;
    use whaleit_storage_sqlite::taxonomies::TaxonomyRepository;
    use whaleit_storage_sqlite::db::{create_pool, run_migrations, write_actor::spawn_writer};

    let pool = create_pool(":memory:").expect("Failed to create SQLite pool");
    run_migrations(":memory:").expect("Failed to run SQLite migrations");

    let writer = spawn_writer((*pool).clone()).expect("Failed to spawn writer actor");

    SqliteRepos {
        accounts: Arc::new(AccountRepository::new(pool.clone(), writer.clone())),
        assets: Arc::new(AssetRepository::new(pool.clone(), writer.clone())),
        fx: Arc::new(FxRepository::new(pool.clone(), writer.clone())),
        goals: Arc::new(GoalRepository::new(pool.clone(), writer.clone())),
        health: Arc::new(HealthDismissalRepository::new(pool.clone(), writer.clone())),
        limits: Arc::new(ContributionLimitRepository::new(pool.clone(), writer.clone())),
        market_data: Arc::new(MarketDataRepository::new(pool.clone(), writer.clone())),
        settings: Arc::new(SettingsRepository::new(pool.clone(), writer.clone())),
        snapshots: Arc::new(SnapshotRepository::new(pool.clone(), writer.clone())),
        taxonomies: Arc::new(TaxonomyRepository::new(pool.clone(), writer.clone())),
        valuations: Arc::new(ValuationRepository::new(pool.clone(), writer.clone())),
    }
}

/// Creates a PostgreSQL repository for testing.
///
/// Requires DATABASE_URL environment variable to be set (e.g., from CI).
async fn create_pg_repo() -> Result<PgRepos, Box<dyn std::error::Error>> {
    use whaleit_storage_postgres::PgAccountRepository;
    use whaleit_storage_postgres::PgAssetRepository;
    use whaleit_storage_postgres::limits::PgContributionLimitRepository;
    use whaleit_storage_postgres::PgFxRepository;
    use whaleit_storage_postgres::goals::PgGoalRepository;
    use whaleit_storage_postgres::health::PgHealthDismissalRepository;
    use whaleit_storage_postgres::market_data::PgMarketDataRepository;
    use whaleit_storage_postgres::PgSettingsRepository;
    use whaleit_storage_postgres::PgSnapshotRepository;
    use whaleit_storage_postgres::taxonomies::PgTaxonomyRepository;
    use whaleit_storage_postgres::PgValuationRepository;
    use whaleit_storage_postgres::db::{create_pool, run_migrations};

    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL environment variable not set")?;

    let pool = create_pool(&database_url, 8)?;
    run_migrations(&database_url).await?;

    Ok(PgRepos {
        accounts: Arc::new(PgAccountRepository::new(pool.clone())),
        assets: Arc::new(PgAssetRepository::new(pool.clone())),
        fx: Arc::new(PgFxRepository::new(pool.clone())),
        goals: Arc::new(PgGoalRepository::new(pool.clone())),
        health: Arc::new(PgHealthDismissalRepository::new(pool.clone())),
        limits: Arc::new(PgContributionLimitRepository::new(pool.clone())),
        market_data: Arc::new(PgMarketDataRepository::new(pool.clone())),
        settings: Arc::new(PgSettingsRepository::new(pool.clone())),
        snapshots: Arc::new(PgSnapshotRepository::new(pool.clone())),
        taxonomies: Arc::new(PgTaxonomyRepository::new(pool.clone())),
        valuations: Arc::new(PgValuationRepository::new(pool.clone())),
    })
}

/// Collection of SQLite repository instances.
struct SqliteRepos {
    accounts: Arc<dyn AccountRepositoryTrait>,
    assets: Arc<dyn AssetRepositoryTrait>,
    fx: Arc<dyn FxRepositoryTrait>,
    goals: Arc<dyn GoalRepositoryTrait>,
    health: Arc<dyn HealthDismissalStore>,
    limits: Arc<dyn ContributionLimitRepositoryTrait>,
    market_data: Arc<dyn QuoteStore>,
    settings: Arc<dyn SettingsRepositoryTrait>,
    snapshots: Arc<dyn SnapshotRepositoryTrait>,
    taxonomies: Arc<dyn TaxonomyRepositoryTrait>,
    valuations: Arc<dyn ValuationRepositoryTrait>,
}

/// Collection of PostgreSQL repository instances.
struct PgRepos {
    accounts: Arc<dyn AccountRepositoryTrait>,
    assets: Arc<dyn AssetRepositoryTrait>,
    fx: Arc<dyn FxRepositoryTrait>,
    goals: Arc<dyn GoalRepositoryTrait>,
    health: Arc<dyn HealthDismissalStore>,
    limits: Arc<dyn ContributionLimitRepositoryTrait>,
    market_data: Arc<dyn QuoteStore>,
    settings: Arc<dyn SettingsRepositoryTrait>,
    snapshots: Arc<dyn SnapshotRepositoryTrait>,
    taxonomies: Arc<dyn TaxonomyRepositoryTrait>,
    valuations: Arc<dyn ValuationRepositoryTrait>,
}

/// Asserts that two Account structs are equal.
fn assert_accounts_equal(a: &whaleit_core::accounts::Account, b: &whaleit_core::accounts::Account) {
    assert_eq!(a.id, b.id, "Account IDs differ");
    assert_eq!(a.name, b.name, "Account names differ");
    assert_eq!(a.account_type, b.account_type, "Account types differ");
    assert_eq!(a.group, b.group, "Account groups differ");
    assert_eq!(a.currency, b.currency, "Account currencies differ");
    assert_eq!(a.is_default, b.is_default, "Account is_default differs");
    assert_eq!(a.is_active, b.is_active, "Account is_active differs");
    assert_eq!(a.is_archived, b.is_archived, "Account is_archived differs");
    assert_eq!(a.tracking_mode, b.tracking_mode, "Account tracking_mode differs");
    // Note: created_at and updated_at may differ slightly between engines, so we don't assert them
    assert_eq!(a.platform_id, b.platform_id, "Account platform_id differs");
    assert_eq!(a.account_number, b.account_number, "Account account_number differs");
    assert_eq!(a.provider, b.provider, "Account provider differs");
    assert_eq!(a.provider_account_id, b.provider_account_id, "Account provider_account_id differs");
    assert_eq!(a.meta, b.meta, "Account meta differs");
}

#[ignore]
#[tokio::test]
async fn parity_account_create() -> Result<(), Box<dyn std::error::Error>> {
    let sqlite_repo = create_sqlite_repo();
    let pg_repo = create_pg_repo().await?;

    // Create identical account in both engines
    let new_account = NewAccount {
        id: None,
        name: "Test Investment Account".to_string(),
        account_type: "INVESTMENT".to_string(),
        group: Some("Brokerage".to_string()),
        currency: "USD".to_string(),
        is_default: true,
        is_active: true,
        is_archived: false,
        tracking_mode: TrackingMode::Transactions,
        platform_id: Some("TEST_PLATFORM".to_string()),
        account_number: Some("12345678".to_string()),
        meta: Some(r#"{"test": "data"}"#.to_string()),
        provider: Some("TEST_PROVIDER".to_string()),
        provider_account_id: Some("provider_123".to_string()),
    };

    let sqlite_account = sqlite_repo.accounts.create(new_account.clone()).await?;
    let pg_account = pg_repo.accounts.create(new_account).await?;

    assert_accounts_equal(&sqlite_account, &pg_account);

    Ok(())
}

#[ignore]
#[tokio::test]
async fn parity_account_update() -> Result<(), Box<dyn std::error::Error>> {
    let sqlite_repo = create_sqlite_repo();
    let pg_repo = create_pg_repo().await?;

    // Create initial account
    let new_account = NewAccount {
        id: None,
        name: "Original Name".to_string(),
        account_type: "INVESTMENT".to_string(),
        group: None,
        currency: "USD".to_string(),
        is_default: false,
        is_active: true,
        is_archived: false,
        tracking_mode: TrackingMode::Transactions,
        platform_id: None,
        account_number: None,
        meta: None,
        provider: None,
        provider_account_id: None,
    };

    let sqlite_account = sqlite_repo.accounts.create(new_account.clone()).await?;
    let pg_account = pg_repo.accounts.create(new_account).await?;

    let sqlite_update = AccountUpdate {
        id: Some(sqlite_account.id.clone()),
        name: "Updated Name".to_string(),
        account_type: "INVESTMENT".to_string(),
        group: Some("Updated Group".to_string()),
        is_default: true,
        is_active: false,
        is_archived: Some(true),
        tracking_mode: Some(TrackingMode::Holdings),
        platform_id: Some("NEW_PLATFORM".to_string()),
        account_number: Some("99999999".to_string()),
        meta: Some(r#"{"updated": "metadata"}"#.to_string()),
        provider: Some("NEW_PROVIDER".to_string()),
        provider_account_id: Some("new_provider_456".to_string()),
    };
    let pg_update = AccountUpdate {
        id: Some(pg_account.id.clone()),
        ..sqlite_update.clone()
    };

    let sqlite_updated = sqlite_repo.accounts.update(sqlite_update).await?;
    let pg_updated = pg_repo.accounts.update(pg_update).await?;

    assert_accounts_equal(&sqlite_updated, &pg_updated);

    Ok(())
}

#[ignore]
#[tokio::test]
async fn parity_account_list() -> Result<(), Box<dyn std::error::Error>> {
    let sqlite_repo = create_sqlite_repo();
    let pg_repo = create_pg_repo().await?;

    // Create multiple accounts with different properties
    let accounts = vec![
        NewAccount {
            id: None,
            name: "Account 1".to_string(),
            account_type: "INVESTMENT".to_string(),
            group: None,
            currency: "USD".to_string(),
            is_default: true,
            is_active: true,
            is_archived: false,
            tracking_mode: TrackingMode::Transactions,
            platform_id: None,
            account_number: None,
            meta: None,
            provider: None,
            provider_account_id: None,
        },
        NewAccount {
            id: None,
            name: "Account 2".to_string(),
            account_type: "INVESTMENT".to_string(),
            group: None,
            currency: "EUR".to_string(),
            is_default: false,
            is_active: false,
            is_archived: false,
            tracking_mode: TrackingMode::Holdings,
            platform_id: None,
            account_number: None,
            meta: None,
            provider: None,
            provider_account_id: None,
        },
        NewAccount {
            id: None,
            name: "Account 3".to_string(),
            account_type: "INVESTMENT".to_string(),
            group: None,
            currency: "GBP".to_string(),
            is_default: false,
            is_active: true,
            is_archived: true,
            tracking_mode: TrackingMode::Transactions,
            platform_id: None,
            account_number: None,
            meta: None,
            provider: None,
            provider_account_id: None,
        },
    ];

    for account in accounts {
        let _ = sqlite_repo.accounts.create(account.clone()).await?;
        let _ = pg_repo.accounts.create(account).await?;
    }

    // Test listing all accounts
    let sqlite_list = sqlite_repo.accounts.list(None, None, None).await?;
    let pg_list = pg_repo.accounts.list(None, None, None).await?;

    assert_eq!(sqlite_list.len(), pg_list.len(), "Account count differs");
    assert_eq!(sqlite_list.len(), 3, "Expected 3 accounts");

    // Test filtering by is_active
    let sqlite_active = sqlite_repo.accounts.list(Some(true), None, None).await?;
    let pg_active = pg_repo.accounts.list(Some(true), None, None).await?;

    assert_eq!(sqlite_active.len(), pg_active.len());
    assert_eq!(sqlite_active.len(), 2, "Expected 2 active accounts");

    // Test filtering by is_archived
    let sqlite_archived = sqlite_repo.accounts.list(None, Some(true), None).await?;
    let pg_archived = pg_repo.accounts.list(None, Some(true), None).await?;

    assert_eq!(sqlite_archived.len(), pg_archived.len());
    assert_eq!(sqlite_archived.len(), 1, "Expected 1 archived account");

    // Verify ordering (should be: is_active desc, is_archived asc, name asc)
    assert_eq!(sqlite_list[0].name, "Account 1");
    assert_eq!(pg_list[0].name, "Account 1");
    assert_eq!(sqlite_list[1].name, "Account 3");
    assert_eq!(pg_list[1].name, "Account 3");
    assert_eq!(sqlite_list[2].name, "Account 2");
    assert_eq!(pg_list[2].name, "Account 2");

    Ok(())
}

#[ignore]
#[tokio::test]
async fn parity_fx_rate() -> Result<(), Box<dyn std::error::Error>> {
    let sqlite_repo = create_sqlite_repo();
    let pg_repo = create_pg_repo().await?;

    // Add an exchange rate
    let now = Utc::now();
    let fx_rate = NewExchangeRate {
        from_currency: "USD".to_string(),
        to_currency: "EUR".to_string(),
        rate: rust_decimal::Decimal::new(92, 2), // 0.92
        source: "test".to_string(),
    };

    let _sqlite_rate = sqlite_repo.fx.save_exchange_rate(ExchangeRate {
        id: uuid::Uuid::new_v4().to_string(),
        from_currency: fx_rate.from_currency.clone(),
        to_currency: fx_rate.to_currency.clone(),
        rate: fx_rate.rate,
        source: fx_rate.source.clone(),
        timestamp: now,
    }).await?;
    let _pg_rate = pg_repo.fx.save_exchange_rate(ExchangeRate {
        id: uuid::Uuid::new_v4().to_string(),
        from_currency: fx_rate.from_currency.clone(),
        to_currency: fx_rate.to_currency.clone(),
        rate: fx_rate.rate,
        source: fx_rate.source.clone(),
        timestamp: now,
    }).await?;

    // Retrieve the rate
    let sqlite_result = sqlite_repo.fx.get_latest_exchange_rate("USD", "EUR").await?;
    let pg_result = pg_repo.fx.get_latest_exchange_rate("USD", "EUR").await?;

    assert_eq!(sqlite_result.is_some(), pg_result.is_some());
    assert!(sqlite_result.is_some(), "FX rate should exist");
    assert!(pg_result.is_some(), "FX rate should exist");

    let sqlite_exchange_rate = sqlite_result.unwrap();
    let pg_exchange_rate = pg_result.unwrap();

    assert_eq!(sqlite_exchange_rate.from_currency, pg_exchange_rate.from_currency);
    assert_eq!(sqlite_exchange_rate.to_currency, pg_exchange_rate.to_currency);
    // Compare rates with some tolerance for floating point
    assert!(
        (sqlite_exchange_rate.rate - pg_exchange_rate.rate).abs() < rust_decimal::Decimal::new(1, 4),
        "FX rates differ significantly"
    );

    Ok(())
}

#[ignore]
#[tokio::test]
async fn parity_settings_update() -> Result<(), Box<dyn std::error::Error>> {
    let sqlite_repo = create_sqlite_repo();
    let pg_repo = create_pg_repo().await?;

    // Update a setting
    let key = "test_setting_key";
    let value = "test_setting_value";

    sqlite_repo.settings.update_setting(key, value).await?;
    pg_repo.settings.update_setting(key, value).await?;

    // Get the setting back
    let sqlite_result = sqlite_repo.settings.get_setting(key).await?;
    let pg_result = pg_repo.settings.get_setting(key).await?;

    assert_eq!(sqlite_result, pg_result);
    assert_eq!(sqlite_result, "test_setting_value".to_string());

    // Update the setting
    let new_value = "updated_value";
    sqlite_repo.settings.update_setting(key, new_value).await?;
    pg_repo.settings.update_setting(key, new_value).await?;

    let sqlite_updated = sqlite_repo.settings.get_setting(key).await?;
    let pg_updated = pg_repo.settings.get_setting(key).await?;

    assert_eq!(sqlite_updated, pg_updated);
    assert_eq!(sqlite_updated, "updated_value".to_string());

    Ok(())
}

#[ignore]
#[tokio::test]
async fn parity_account_get_by_id() -> Result<(), Box<dyn std::error::Error>> {
    let sqlite_repo = create_sqlite_repo();
    let pg_repo = create_pg_repo().await?;

    // Create an account
    let new_account = NewAccount {
        id: None,
        name: "Test Account".to_string(),
        account_type: "INVESTMENT".to_string(),
        group: None,
        currency: "USD".to_string(),
        is_default: false,
        is_active: true,
        is_archived: false,
        tracking_mode: TrackingMode::Transactions,
        platform_id: None,
        account_number: None,
        meta: None,
        provider: None,
        provider_account_id: None,
    };

    let sqlite_account = sqlite_repo.accounts.create(new_account.clone()).await?;
    let pg_account = pg_repo.accounts.create(new_account).await?;

    // Get by ID
    let sqlite_result = sqlite_repo.accounts.get_by_id(&sqlite_account.id).await?;
    let pg_result = pg_repo.accounts.get_by_id(&pg_account.id).await?;

    assert_accounts_equal(&sqlite_result, &pg_result);

    Ok(())
}

#[ignore]
#[tokio::test]
async fn parity_account_delete() -> Result<(), Box<dyn std::error::Error>> {
    let sqlite_repo = create_sqlite_repo();
    let pg_repo = create_pg_repo().await?;

    // Create and delete accounts
    for i in 1..=3 {
        let new_account = NewAccount {
            id: None,
            name: format!("Account {}", i),
            account_type: "INVESTMENT".to_string(),
            group: None,
            currency: "USD".to_string(),
            is_default: false,
            is_active: true,
            is_archived: false,
            tracking_mode: TrackingMode::Transactions,
            platform_id: None,
            account_number: None,
            meta: None,
            provider: None,
            provider_account_id: None,
        };

        let sqlite_account = sqlite_repo.accounts.create(new_account.clone()).await?;
        let pg_account = pg_repo.accounts.create(new_account.clone()).await?;

        // Delete from both
        let sqlite_count = sqlite_repo.accounts.delete(&sqlite_account.id).await?;
        let pg_count = pg_repo.accounts.delete(&pg_account.id).await?;

        assert_eq!(sqlite_count, 1);
        assert_eq!(pg_count, 1);
        assert_eq!(sqlite_count, pg_count);
    }

    Ok(())
}

#[ignore]
#[tokio::test]
async fn parity_settings_get_settings() -> Result<(), Box<dyn std::error::Error>> {
    let sqlite_repo = create_sqlite_repo();
    let pg_repo = create_pg_repo().await?;

    // Get settings (should be defaults)
    let sqlite_settings = sqlite_repo.settings.get_settings().await?;
    let pg_settings = pg_repo.settings.get_settings().await?;

    assert_eq!(sqlite_settings.theme, pg_settings.theme);
    assert_eq!(sqlite_settings.font, pg_settings.font);
    assert_eq!(sqlite_settings.base_currency, pg_settings.base_currency);
    assert_eq!(sqlite_settings.timezone, pg_settings.timezone);
    assert_eq!(sqlite_settings.onboarding_completed, pg_settings.onboarding_completed);

    Ok(())
}

#[ignore]
#[tokio::test]
async fn parity_asset_create() -> Result<(), Box<dyn std::error::Error>> {
    let sqlite_repo = create_sqlite_repo();
    let pg_repo = create_pg_repo().await?;

    let new_asset = NewAsset {
        id: None,
        kind: AssetKind::Investment,
        name: Some("Apple Inc.".to_string()),
        display_code: Some("AAPL".to_string()),
        is_active: true,
        quote_mode: QuoteMode::Market,
        quote_ccy: "USD".to_string(),
        instrument_type: Some(InstrumentType::Equity),
        instrument_symbol: Some("AAPL".to_string()),
        instrument_exchange_mic: Some("XNAS".to_string()),
        provider_config: None,
        notes: None,
        metadata: None,
    };

    let sqlite_asset = sqlite_repo.assets.create(new_asset.clone()).await?;
    let pg_asset = pg_repo.assets.create(new_asset).await?;

    assert_eq!(sqlite_asset.kind, pg_asset.kind, "Asset kinds differ");
    assert_eq!(sqlite_asset.name, pg_asset.name, "Asset names differ");
    assert_eq!(sqlite_asset.display_code, pg_asset.display_code, "Asset display_codes differ");
    assert_eq!(sqlite_asset.is_active, pg_asset.is_active, "Asset is_active differs");
    assert_eq!(sqlite_asset.quote_mode, pg_asset.quote_mode, "Asset quote_modes differ");
    assert_eq!(sqlite_asset.quote_ccy, pg_asset.quote_ccy, "Asset quote_ccy differs");
    assert_eq!(sqlite_asset.instrument_type, pg_asset.instrument_type, "Asset instrument_types differ");
    assert_eq!(sqlite_asset.instrument_symbol, pg_asset.instrument_symbol, "Asset instrument_symbols differ");
    assert_eq!(sqlite_asset.instrument_exchange_mic, pg_asset.instrument_exchange_mic, "Asset instrument_exchange_mics differ");

    Ok(())
}

#[ignore]
#[tokio::test]
async fn parity_asset_get_by_id() -> Result<(), Box<dyn std::error::Error>> {
    let sqlite_repo = create_sqlite_repo();
    let pg_repo = create_pg_repo().await?;

    let new_asset = NewAsset {
        id: None,
        kind: AssetKind::Investment,
        name: Some("Microsoft Corp.".to_string()),
        display_code: Some("MSFT".to_string()),
        is_active: true,
        quote_mode: QuoteMode::Market,
        quote_ccy: "USD".to_string(),
        instrument_type: Some(InstrumentType::Equity),
        instrument_symbol: Some("MSFT".to_string()),
        instrument_exchange_mic: Some("XNAS".to_string()),
        provider_config: None,
        notes: None,
        metadata: None,
    };

    let sqlite_asset = sqlite_repo.assets.create(new_asset.clone()).await?;
    let pg_asset = pg_repo.assets.create(new_asset).await?;

    let sqlite_result = sqlite_repo.assets.get_by_id(&sqlite_asset.id).await?;
    let pg_result = pg_repo.assets.get_by_id(&pg_asset.id).await?;

    assert_eq!(sqlite_result.name, pg_result.name, "Asset names differ");
    assert_eq!(sqlite_result.instrument_symbol, pg_result.instrument_symbol, "Asset instrument_symbols differ");
    assert_eq!(sqlite_result.quote_ccy, pg_result.quote_ccy, "Asset quote_ccy differs");

    Ok(())
}

#[ignore]
#[tokio::test]
async fn parity_goal_create() -> Result<(), Box<dyn std::error::Error>> {
    let sqlite_repo = create_sqlite_repo();
    let pg_repo = create_pg_repo().await?;

    let new_goal = NewGoal {
        id: None,
        title: "Emergency Fund".to_string(),
        description: Some("Build 6 month emergency fund".to_string()),
        target_amount: 50000.0,
        is_achieved: false,
    };

    let sqlite_goal = sqlite_repo.goals.insert_new_goal(new_goal.clone()).await?;
    let pg_goal = pg_repo.goals.insert_new_goal(new_goal).await?;

    assert_eq!(sqlite_goal.title, pg_goal.title, "Goal titles differ");
    assert_eq!(sqlite_goal.description, pg_goal.description, "Goal descriptions differ");
    assert_eq!(sqlite_goal.target_amount, pg_goal.target_amount, "Goal target_amounts differ");
    assert_eq!(sqlite_goal.is_achieved, pg_goal.is_achieved, "Goal is_achieved differs");

    Ok(())
}

#[ignore]
#[tokio::test]
async fn parity_health_dismissal() -> Result<(), Box<dyn std::error::Error>> {
    let sqlite_repo = create_sqlite_repo();
    let pg_repo = create_pg_repo().await?;

    let dismissal = IssueDismissal::new("price_stale:AAPL", "abc123hash");

    sqlite_repo.health.save_dismissal(&dismissal).await?;
    pg_repo.health.save_dismissal(&dismissal).await?;

    let sqlite_result = sqlite_repo.health.get_dismissal("price_stale:AAPL").await?;
    let pg_result = pg_repo.health.get_dismissal("price_stale:AAPL").await?;

    assert!(sqlite_result.is_some(), "SQLite dismissal should exist");
    assert!(pg_result.is_some(), "PG dismissal should exist");

    let sqlite_dismissal = sqlite_result.unwrap();
    let pg_dismissal = pg_result.unwrap();

    assert_eq!(sqlite_dismissal.issue_id, pg_dismissal.issue_id, "Dismissal issue_ids differ");
    assert_eq!(sqlite_dismissal.data_hash, pg_dismissal.data_hash, "Dismissal data_hashes differ");

    let sqlite_all = sqlite_repo.health.get_dismissals().await?;
    let pg_all = pg_repo.health.get_dismissals().await?;

    assert_eq!(sqlite_all.len(), pg_all.len(), "Dismissal counts differ");

    Ok(())
}

#[ignore]
#[tokio::test]
async fn parity_limit_create() -> Result<(), Box<dyn std::error::Error>> {
    let sqlite_repo = create_sqlite_repo();
    let pg_repo = create_pg_repo().await?;

    let new_limit = NewContributionLimit {
        id: None,
        group_name: "ISA".to_string(),
        contribution_year: 2025,
        limit_amount: 20000.0,
        account_ids: None,
        start_date: None,
        end_date: None,
    };

    let sqlite_limit = sqlite_repo.limits.create_contribution_limit(new_limit.clone()).await?;
    let pg_limit = pg_repo.limits.create_contribution_limit(new_limit).await?;

    assert_eq!(sqlite_limit.group_name, pg_limit.group_name, "Limit group_names differ");
    assert_eq!(sqlite_limit.contribution_year, pg_limit.contribution_year, "Limit contribution_years differ");
    assert_eq!(sqlite_limit.limit_amount, pg_limit.limit_amount, "Limit limit_amounts differ");
    assert_eq!(sqlite_limit.account_ids, pg_limit.account_ids, "Limit account_ids differ");

    Ok(())
}

#[ignore]
#[tokio::test]
async fn parity_taxonomy_create() -> Result<(), Box<dyn std::error::Error>> {
    let sqlite_repo = create_sqlite_repo();
    let pg_repo = create_pg_repo().await?;

    let new_taxonomy = NewTaxonomy {
        id: None,
        name: "Asset Classes".to_string(),
        color: "#ff0000".to_string(),
        description: Some("Major asset classes".to_string()),
        is_system: false,
        is_single_select: true,
        sort_order: 1,
    };

    let sqlite_taxonomy = sqlite_repo.taxonomies.create_taxonomy(new_taxonomy.clone()).await?;
    let pg_taxonomy = pg_repo.taxonomies.create_taxonomy(new_taxonomy).await?;

    assert_eq!(sqlite_taxonomy.name, pg_taxonomy.name, "Taxonomy names differ");
    assert_eq!(sqlite_taxonomy.color, pg_taxonomy.color, "Taxonomy colors differ");
    assert_eq!(sqlite_taxonomy.description, pg_taxonomy.description, "Taxonomy descriptions differ");
    assert_eq!(sqlite_taxonomy.is_system, pg_taxonomy.is_system, "Taxonomy is_system differs");
    assert_eq!(sqlite_taxonomy.is_single_select, pg_taxonomy.is_single_select, "Taxonomy is_single_select differs");
    assert_eq!(sqlite_taxonomy.sort_order, pg_taxonomy.sort_order, "Taxonomy sort_orders differ");

    Ok(())
}

#[ignore]
#[tokio::test]
async fn parity_snapshot_save_and_get() -> Result<(), Box<dyn std::error::Error>> {
    let sqlite_repo = create_sqlite_repo();
    let pg_repo = create_pg_repo().await?;

    let account_id = uuid::Uuid::new_v4().to_string();
    let snapshot_date = chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
    let snapshot_id = whaleit_core::portfolio::snapshot::AccountStateSnapshot::stable_id(&account_id, snapshot_date);

    let snapshot = AccountStateSnapshot {
        id: snapshot_id.clone(),
        account_id: account_id.clone(),
        snapshot_date,
        currency: "USD".to_string(),
        positions: std::collections::HashMap::new(),
        cash_balances: std::collections::HashMap::new(),
        cost_basis: rust_decimal::Decimal::ZERO,
        net_contribution: rust_decimal::Decimal::new(5000, 0),
        net_contribution_base: rust_decimal::Decimal::new(5000, 0),
        cash_total_account_currency: rust_decimal::Decimal::new(1000, 0),
        cash_total_base_currency: rust_decimal::Decimal::new(1000, 0),
        calculated_at: chrono::Utc::now().naive_utc(),
        source: SnapshotSource::Calculated,
    };

    sqlite_repo.snapshots.save_snapshots(&[snapshot.clone()]).await?;
    pg_repo.snapshots.save_snapshots(&[snapshot.clone()]).await?;

    let sqlite_results = sqlite_repo.snapshots.get_snapshots_by_account(&account_id, None, None).await?;
    let pg_results = pg_repo.snapshots.get_snapshots_by_account(&account_id, None, None).await?;

    assert_eq!(sqlite_results.len(), 1, "SQLite should have 1 snapshot");
    assert_eq!(pg_results.len(), 1, "PG should have 1 snapshot");

    let sqlite_snap = &sqlite_results[0];
    let pg_snap = &pg_results[0];

    assert_eq!(sqlite_snap.account_id, pg_snap.account_id, "Snapshot account_ids differ");
    assert_eq!(sqlite_snap.snapshot_date, pg_snap.snapshot_date, "Snapshot dates differ");
    assert_eq!(sqlite_snap.currency, pg_snap.currency, "Snapshot currencies differ");
    assert_eq!(sqlite_snap.net_contribution, pg_snap.net_contribution, "Snapshot net_contributions differ");
    assert_eq!(sqlite_snap.source, pg_snap.source, "Snapshot sources differ");

    Ok(())
}

#[ignore]
#[tokio::test]
async fn parity_valuation_save_and_get() -> Result<(), Box<dyn std::error::Error>> {
    let sqlite_repo = create_sqlite_repo();
    let pg_repo = create_pg_repo().await?;

    let account_id = uuid::Uuid::new_v4().to_string();
    let valuation_date = chrono::NaiveDate::from_ymd_opt(2025, 3, 1).unwrap();

    let valuation = DailyAccountValuation {
        id: uuid::Uuid::new_v4().to_string(),
        account_id: account_id.clone(),
        valuation_date,
        account_currency: "USD".to_string(),
        base_currency: "USD".to_string(),
        fx_rate_to_base: rust_decimal::Decimal::ONE,
        cash_balance: rust_decimal::Decimal::new(2500, 2),
        investment_market_value: rust_decimal::Decimal::new(7500, 2),
        total_value: rust_decimal::Decimal::new(10000, 2),
        cost_basis: rust_decimal::Decimal::new(8000, 2),
        net_contribution: rust_decimal::Decimal::new(8000, 2),
        calculated_at: chrono::Utc::now(),
    };

    sqlite_repo.valuations.save_valuations(&[valuation.clone()]).await?;
    pg_repo.valuations.save_valuations(&[valuation.clone()]).await?;

    let sqlite_results = sqlite_repo.valuations.get_historical_valuations(&account_id, None, None).await?;
    let pg_results = pg_repo.valuations.get_historical_valuations(&account_id, None, None).await?;

    assert_eq!(sqlite_results.len(), 1, "SQLite should have 1 valuation");
    assert_eq!(pg_results.len(), 1, "PG should have 1 valuation");

    let sqlite_val = &sqlite_results[0];
    let pg_val = &pg_results[0];

    assert_eq!(sqlite_val.account_id, pg_val.account_id, "Valuation account_ids differ");
    assert_eq!(sqlite_val.valuation_date, pg_val.valuation_date, "Valuation dates differ");
    assert_eq!(sqlite_val.account_currency, pg_val.account_currency, "Valuation account_currencies differ");
    assert_eq!(sqlite_val.base_currency, pg_val.base_currency, "Valuation base_currencies differ");
    assert_eq!(sqlite_val.fx_rate_to_base, pg_val.fx_rate_to_base, "Valuation fx_rate_to_base differs");
    assert_eq!(sqlite_val.cash_balance, pg_val.cash_balance, "Valuation cash_balances differ");
    assert_eq!(sqlite_val.total_value, pg_val.total_value, "Valuation total_values differ");

    Ok(())
}

#[ignore]
#[tokio::test]
async fn parity_market_data_quote() -> Result<(), Box<dyn std::error::Error>> {
    let sqlite_repo = create_sqlite_repo();
    let pg_repo = create_pg_repo().await?;

    let asset_id = format!("SEC:AAPL:{}", uuid::Uuid::new_v4());
    let quote = Quote {
        id: uuid::Uuid::new_v4().to_string(),
        asset_id: asset_id.clone(),
        timestamp: chrono::Utc::now(),
        open: rust_decimal::Decimal::new(15025, 2),
        high: rust_decimal::Decimal::new(15100, 2),
        low: rust_decimal::Decimal::new(14980, 2),
        close: rust_decimal::Decimal::new(15050, 2),
        adjclose: rust_decimal::Decimal::new(15050, 2),
        volume: rust_decimal::Decimal::new(5000000, 0),
        currency: "USD".to_string(),
        data_source: "YAHOO".to_string(),
        created_at: chrono::Utc::now(),
        notes: None,
    };

    let sqlite_saved = sqlite_repo.market_data.save_quote(&quote).await?;
    let pg_saved = pg_repo.market_data.save_quote(&quote).await?;

    assert_eq!(sqlite_saved.asset_id, pg_saved.asset_id, "Quote asset_ids differ");
    assert_eq!(sqlite_saved.close, pg_saved.close, "Quote closes differ");
    assert_eq!(sqlite_saved.currency, pg_saved.currency, "Quote currencies differ");
    assert_eq!(sqlite_saved.data_source, pg_saved.data_source, "Quote data_sources differ");

    let sqlite_latest = sqlite_repo.market_data.get_latest_quote(&asset_id).await?;
    let pg_latest = pg_repo.market_data.get_latest_quote(&asset_id).await?;

    assert_eq!(sqlite_latest.asset_id, pg_latest.asset_id, "Latest quote asset_ids differ");
    assert_eq!(sqlite_latest.close, pg_latest.close, "Latest quote closes differ");
    assert_eq!(sqlite_latest.currency, pg_latest.currency, "Latest quote currencies differ");

    Ok(())
}

