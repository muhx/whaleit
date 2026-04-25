//! Service-level tests for AccountService. Phase 3 focus: D-12 balance
//! auto-bump behavior in update_account.

#[cfg(test)]
mod tests {
    use crate::accounts::{
        accounts_model::{Account, AccountUpdate, NewAccount, TrackingMode},
        accounts_service::AccountService,
        accounts_traits::{AccountRepositoryTrait, AccountServiceTrait},
    };
    use crate::assets::{
        Asset, AssetRepositoryTrait, AssetSpec, EnsureAssetsResult, NewAsset, UpdateAssetProfile,
    };
    use crate::events::NoOpDomainEventSink;
    use crate::fx::{ExchangeRate, FxServiceTrait, NewExchangeRate};
    use crate::quotes::sync_state::{ProviderSyncStats, QuoteSyncState, SyncStateStore};
    use crate::Result;
    use async_trait::async_trait;
    use chrono::NaiveDate;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex, RwLock};

    // -- Mock account repository ----------------------------------------------

    struct MockAccountRepo {
        existing: Account,
        last_update: Mutex<Option<AccountUpdate>>,
    }

    #[async_trait]
    impl AccountRepositoryTrait for MockAccountRepo {
        async fn create(&self, _new_account: NewAccount) -> Result<Account> {
            unimplemented!("not exercised by update_account tests")
        }
        async fn update(&self, account_update: AccountUpdate) -> Result<Account> {
            *self.last_update.lock().unwrap() = Some(account_update.clone());
            // Reflect the update onto the mock "existing" Account for the return value.
            let mut updated = self.existing.clone();
            if let Some(b) = account_update.current_balance {
                updated.current_balance = Some(b);
            }
            if let Some(ts) = account_update.balance_updated_at {
                updated.balance_updated_at = Some(ts);
            }
            Ok(updated)
        }
        async fn delete(&self, _account_id: &str) -> Result<usize> {
            unimplemented!("not exercised by update_account tests")
        }
        async fn get_by_id(&self, _account_id: &str) -> Result<Account> {
            Ok(self.existing.clone())
        }
        async fn list(
            &self,
            _is_active_filter: Option<bool>,
            _is_archived_filter: Option<bool>,
            _account_ids: Option<&[String]>,
        ) -> Result<Vec<Account>> {
            unimplemented!("not exercised by update_account tests")
        }
    }

    // -- Stub FxService ------------------------------------------------------
    // update_account calls register_currency_pair only when currency changes.
    // Tests keep currency stable so this never fires; all methods panic loudly.

    struct StubFx;

    #[async_trait]
    impl FxServiceTrait for StubFx {
        async fn initialize(&self) -> Result<()> {
            unimplemented!("StubFx::initialize")
        }
        async fn get_historical_rates(
            &self,
            _from_currency: &str,
            _to_currency: &str,
            _days: i64,
        ) -> Result<Vec<ExchangeRate>> {
            unimplemented!("StubFx::get_historical_rates")
        }
        async fn get_latest_exchange_rate(
            &self,
            _from_currency: &str,
            _to_currency: &str,
        ) -> Result<Decimal> {
            unimplemented!("StubFx::get_latest_exchange_rate")
        }
        async fn get_exchange_rate_for_date(
            &self,
            _from_currency: &str,
            _to_currency: &str,
            _date: NaiveDate,
        ) -> Result<Decimal> {
            unimplemented!("StubFx::get_exchange_rate_for_date")
        }
        async fn convert_currency(
            &self,
            _amount: Decimal,
            _from_currency: &str,
            _to_currency: &str,
        ) -> Result<Decimal> {
            unimplemented!("StubFx::convert_currency")
        }
        async fn convert_currency_for_date(
            &self,
            _amount: Decimal,
            _from_currency: &str,
            _to_currency: &str,
            _date: NaiveDate,
        ) -> Result<Decimal> {
            unimplemented!("StubFx::convert_currency_for_date")
        }
        async fn get_latest_exchange_rates(&self) -> Result<Vec<ExchangeRate>> {
            unimplemented!("StubFx::get_latest_exchange_rates")
        }
        async fn add_exchange_rate(&self, _new_rate: NewExchangeRate) -> Result<ExchangeRate> {
            unimplemented!("StubFx::add_exchange_rate")
        }
        async fn update_exchange_rate(
            &self,
            _from_currency: &str,
            _to_currency: &str,
            _rate: Decimal,
        ) -> Result<ExchangeRate> {
            unimplemented!("StubFx::update_exchange_rate")
        }
        async fn delete_exchange_rate(&self, _rate_id: &str) -> Result<()> {
            unimplemented!("StubFx::delete_exchange_rate")
        }
        async fn register_currency_pair(
            &self,
            _from_currency: &str,
            _to_currency: &str,
        ) -> Result<()> {
            unimplemented!("StubFx::register_currency_pair")
        }
        async fn register_currency_pair_manual(
            &self,
            _from_currency: &str,
            _to_currency: &str,
        ) -> Result<()> {
            unimplemented!("StubFx::register_currency_pair_manual")
        }
        async fn ensure_fx_pairs(&self, _pairs: Vec<(String, String)>) -> Result<()> {
            unimplemented!("StubFx::ensure_fx_pairs")
        }
    }

    // -- Stub AssetRepository -------------------------------------------------
    // update_account does not touch asset repository at all. All methods panic.

    struct StubAssets;

    #[async_trait]
    impl AssetRepositoryTrait for StubAssets {
        async fn create(&self, _new_asset: NewAsset) -> Result<Asset> {
            unimplemented!("StubAssets::create")
        }
        async fn create_batch(&self, _new_assets: Vec<NewAsset>) -> Result<Vec<Asset>> {
            unimplemented!("StubAssets::create_batch")
        }
        async fn update_profile(
            &self,
            _asset_id: &str,
            _payload: UpdateAssetProfile,
        ) -> Result<Asset> {
            unimplemented!("StubAssets::update_profile")
        }
        async fn update_quote_mode(&self, _asset_id: &str, _quote_mode: &str) -> Result<Asset> {
            unimplemented!("StubAssets::update_quote_mode")
        }
        async fn get_by_id(&self, _asset_id: &str) -> Result<Asset> {
            unimplemented!("StubAssets::get_by_id")
        }
        async fn list(&self) -> Result<Vec<Asset>> {
            unimplemented!("StubAssets::list")
        }
        async fn list_by_asset_ids(&self, _asset_ids: &[String]) -> Result<Vec<Asset>> {
            unimplemented!("StubAssets::list_by_asset_ids")
        }
        async fn delete(&self, _asset_id: &str) -> Result<()> {
            unimplemented!("StubAssets::delete")
        }
        async fn search_by_symbol(&self, _query: &str) -> Result<Vec<Asset>> {
            unimplemented!("StubAssets::search_by_symbol")
        }
        async fn find_by_instrument_key(&self, _instrument_key: &str) -> Result<Option<Asset>> {
            unimplemented!("StubAssets::find_by_instrument_key")
        }
        async fn cleanup_legacy_metadata(&self, _asset_id: &str) -> Result<()> {
            unimplemented!("StubAssets::cleanup_legacy_metadata")
        }
        async fn deactivate(&self, _asset_id: &str) -> Result<()> {
            unimplemented!("StubAssets::deactivate")
        }
        async fn reactivate(&self, _asset_id: &str) -> Result<()> {
            unimplemented!("StubAssets::reactivate")
        }
        async fn copy_user_metadata(&self, _source_id: &str, _target_id: &str) -> Result<()> {
            unimplemented!("StubAssets::copy_user_metadata")
        }
        async fn deactivate_orphaned_investments(&self) -> Result<Vec<String>> {
            unimplemented!("StubAssets::deactivate_orphaned_investments")
        }
    }

    // Silence unused-import warnings for AssetSpec/EnsureAssetsResult — they
    // only appear in the wider AssetServiceTrait surface, not our stub. Keep
    // them imported so the test file documents the asset-domain dependency.
    #[allow(dead_code)]
    fn _doc_asset_types(_spec: AssetSpec, _result: EnsureAssetsResult) {}

    // -- Stub SyncStateStore --------------------------------------------------
    // update_account does not touch sync state store. All methods panic.

    struct StubSyncState;

    #[async_trait]
    impl SyncStateStore for StubSyncState {
        async fn get_provider_sync_stats(&self) -> Result<Vec<ProviderSyncStats>> {
            unimplemented!("StubSyncState::get_provider_sync_stats")
        }
        async fn get_all(&self) -> Result<Vec<QuoteSyncState>> {
            unimplemented!("StubSyncState::get_all")
        }
        async fn get_by_asset_id(&self, _asset_id: &str) -> Result<Option<QuoteSyncState>> {
            unimplemented!("StubSyncState::get_by_asset_id")
        }
        async fn get_by_asset_ids(
            &self,
            _asset_ids: &[String],
        ) -> Result<HashMap<String, QuoteSyncState>> {
            unimplemented!("StubSyncState::get_by_asset_ids")
        }
        async fn get_active_assets(&self) -> Result<Vec<QuoteSyncState>> {
            unimplemented!("StubSyncState::get_active_assets")
        }
        async fn get_assets_needing_sync(
            &self,
            _grace_period_days: i64,
        ) -> Result<Vec<QuoteSyncState>> {
            unimplemented!("StubSyncState::get_assets_needing_sync")
        }
        async fn upsert(&self, _state: &QuoteSyncState) -> Result<QuoteSyncState> {
            unimplemented!("StubSyncState::upsert")
        }
        async fn upsert_batch(&self, _states: &[QuoteSyncState]) -> Result<usize> {
            unimplemented!("StubSyncState::upsert_batch")
        }
        async fn update_after_sync(&self, _asset_id: &str) -> Result<()> {
            unimplemented!("StubSyncState::update_after_sync")
        }
        async fn update_after_failure(&self, _asset_id: &str, _error: &str) -> Result<()> {
            unimplemented!("StubSyncState::update_after_failure")
        }
        async fn mark_inactive(&self, _asset_id: &str, _closed_date: NaiveDate) -> Result<()> {
            unimplemented!("StubSyncState::mark_inactive")
        }
        async fn mark_active(&self, _asset_id: &str) -> Result<()> {
            unimplemented!("StubSyncState::mark_active")
        }
        async fn delete(&self, _asset_id: &str) -> Result<()> {
            unimplemented!("StubSyncState::delete")
        }
        async fn delete_all(&self) -> Result<usize> {
            unimplemented!("StubSyncState::delete_all")
        }
        async fn mark_profile_enriched(&self, _asset_id: &str) -> Result<()> {
            unimplemented!("StubSyncState::mark_profile_enriched")
        }
        async fn get_assets_needing_profile_enrichment(&self) -> Result<Vec<QuoteSyncState>> {
            unimplemented!("StubSyncState::get_assets_needing_profile_enrichment")
        }
        async fn get_with_errors(&self) -> Result<Vec<QuoteSyncState>> {
            unimplemented!("StubSyncState::get_with_errors")
        }
    }

    // -- Builders ------------------------------------------------------------

    fn make_service(existing: Account) -> (AccountService, Arc<MockAccountRepo>) {
        let repo = Arc::new(MockAccountRepo {
            existing,
            last_update: Mutex::new(None),
        });
        let service = AccountService::new(
            repo.clone(),
            Arc::new(StubFx),
            Arc::new(RwLock::new("USD".to_string())),
            Arc::new(NoOpDomainEventSink),
            Arc::new(StubAssets),
            Arc::new(StubSyncState),
        );
        (service, repo)
    }

    fn existing_cc(current: Option<Decimal>) -> Account {
        Account {
            id: "acc-1".to_string(),
            name: "Card".to_string(),
            account_type: "CREDIT_CARD".to_string(),
            currency: "USD".to_string(),
            is_active: true,
            tracking_mode: TrackingMode::Transactions,
            current_balance: current,
            ..Default::default()
        }
    }

    fn update_with_balance(id: &str, balance: Option<Decimal>) -> AccountUpdate {
        AccountUpdate {
            id: Some(id.to_string()),
            name: "Card".to_string(),
            account_type: "CREDIT_CARD".to_string(),
            group: None,
            is_default: false,
            is_active: true,
            platform_id: None,
            account_number: None,
            meta: None,
            provider: None,
            provider_account_id: None,
            is_archived: None,
            tracking_mode: None,
            institution: None,
            opening_balance: None,
            current_balance: balance,
            balance_updated_at: None,
            credit_limit: None,
            statement_cycle_day: None,
            statement_balance: None,
            minimum_payment: None,
            statement_due_date: None,
            reward_points_balance: None,
            cashback_balance: None,
        }
    }

    // -- Tests ---------------------------------------------------------------

    #[tokio::test]
    async fn test_update_bumps_balance_timestamp() {
        let (service, repo) = make_service(existing_cc(Some(dec!(100))));
        let update = update_with_balance("acc-1", Some(dec!(200)));
        service.update_account(update).await.unwrap();

        let captured = repo.last_update.lock().unwrap().clone().unwrap();
        assert!(
            captured.balance_updated_at.is_some(),
            "balance_updated_at should auto-stamp when current_balance changes"
        );
    }

    #[tokio::test]
    async fn test_update_no_bump_when_balance_unchanged() {
        let (service, repo) = make_service(existing_cc(Some(dec!(100))));
        let update = update_with_balance("acc-1", Some(dec!(100)));
        service.update_account(update).await.unwrap();

        let captured = repo.last_update.lock().unwrap().clone().unwrap();
        assert!(
            captured.balance_updated_at.is_none(),
            "balance_updated_at should not auto-stamp when current_balance is unchanged"
        );
    }

    #[tokio::test]
    async fn test_update_no_bump_when_no_balance_in_update() {
        let (service, repo) = make_service(existing_cc(Some(dec!(100))));
        let update = update_with_balance("acc-1", None);
        service.update_account(update).await.unwrap();

        let captured = repo.last_update.lock().unwrap().clone().unwrap();
        assert!(
            captured.balance_updated_at.is_none(),
            "balance_updated_at should not auto-stamp when current_balance not supplied"
        );
    }
}
