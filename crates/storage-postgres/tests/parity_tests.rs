//! Parity tests comparing SQLite and PostgreSQL repository behavior.
//!
//! These tests verify that both database engines produce identical results
//! when given the same input data. Tests are marked with `#[ignore]` by default
//! and should be run with `cargo test -p whaleit-storage-postgres -- --ignored`
//! or via CI with PostgreSQL service container.

use std::sync::Arc;

use chrono::Utc;
use whaleit_core::accounts::{AccountRepositoryTrait, AccountUpdate, NewAccount, TrackingMode};
use whaleit_core::fx::{ExchangeRate, FxRepositoryTrait, NewExchangeRate};
use whaleit_core::settings::SettingsRepositoryTrait;

/// Creates an in-memory SQLite repository for testing.
fn create_sqlite_repo() -> SqliteRepos {
    use whaleit_storage_sqlite::accounts::AccountRepository;
    use whaleit_storage_sqlite::fx::FxRepository;
    use whaleit_storage_sqlite::settings::SettingsRepository;
    use whaleit_storage_sqlite::db::{create_pool, run_migrations, write_actor::spawn_writer};

    // Create in-memory SQLite database
    let pool = create_pool(":memory:").expect("Failed to create SQLite pool");
    run_migrations(":memory:").expect("Failed to run SQLite migrations");

    // spawn_writer expects DbPool (not Arc<DbPool>), so we need to clone the inner pool
    // Since pool is Arc<DbPool>, we dereference to get DbPool, then clone it
    let writer = spawn_writer((*pool).clone()).expect("Failed to spawn writer actor");

    SqliteRepos {
        accounts: Arc::new(AccountRepository::new(pool.clone(), writer.clone())),
        fx: Arc::new(FxRepository::new(pool.clone(), writer.clone())),
        settings: Arc::new(SettingsRepository::new(pool.clone(), writer.clone())),
    }
}

/// Creates a PostgreSQL repository for testing.
///
/// Requires DATABASE_URL environment variable to be set (e.g., from CI).
async fn create_pg_repo() -> Result<PgRepos, Box<dyn std::error::Error>> {
    use whaleit_storage_postgres::PgAccountRepository;
    use whaleit_storage_postgres::PgFxRepository;
    use whaleit_storage_postgres::PgSettingsRepository;
    use whaleit_storage_postgres::db::{create_pool, run_migrations};

    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL environment variable not set")?;

    let pool = create_pool(&database_url, 8)?;
    run_migrations(&database_url).await?;

    Ok(PgRepos {
        accounts: Arc::new(PgAccountRepository::new(pool.clone())),
        fx: Arc::new(PgFxRepository::new(pool.clone())),
        settings: Arc::new(PgSettingsRepository::new(pool.clone())),
    })
}

/// Collection of SQLite repository instances.
struct SqliteRepos {
    accounts: Arc<dyn AccountRepositoryTrait>,
    fx: Arc<dyn FxRepositoryTrait>,
    settings: Arc<dyn SettingsRepositoryTrait>,
}

/// Collection of PostgreSQL repository instances.
struct PgRepos {
    accounts: Arc<dyn AccountRepositoryTrait>,
    fx: Arc<dyn FxRepositoryTrait>,
    settings: Arc<dyn SettingsRepositoryTrait>,
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

    // Update both
    let account_update = AccountUpdate {
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

    let sqlite_updated = sqlite_repo.accounts.update(account_update.clone()).await?;
    let pg_updated = pg_repo.accounts.update(account_update).await?;

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

