//! Account service implementation.

use log::{debug, info, warn};
use std::sync::{Arc, RwLock};

use super::accounts_model::{Account, AccountUpdate, NewAccount};
use super::accounts_traits::{AccountRepositoryTrait, AccountServiceTrait};
use crate::assets::AssetRepositoryTrait;
use crate::errors::Result;
use crate::events::{CurrencyChange, DomainEvent, DomainEventSink};
use crate::fx::FxServiceTrait;
use crate::quotes::sync_state::SyncStateStore;

/// Service for managing accounts.
pub struct AccountService {
    repository: Arc<dyn AccountRepositoryTrait>,
    fx_service: Arc<dyn FxServiceTrait>,
    base_currency: Arc<RwLock<String>>,
    event_sink: Arc<dyn DomainEventSink>,
    asset_repository: Arc<dyn AssetRepositoryTrait>,
    sync_state_store: Arc<dyn SyncStateStore>,
}

impl AccountService {
    /// Creates a new AccountService instance.
    pub fn new(
        repository: Arc<dyn AccountRepositoryTrait>,
        fx_service: Arc<dyn FxServiceTrait>,
        base_currency: Arc<RwLock<String>>,
        event_sink: Arc<dyn DomainEventSink>,
        asset_repository: Arc<dyn AssetRepositoryTrait>,
        sync_state_store: Arc<dyn SyncStateStore>,
    ) -> Self {
        Self {
            repository,
            fx_service,
            base_currency,
            event_sink,
            asset_repository,
            sync_state_store,
        }
    }
}

#[async_trait::async_trait]
impl AccountServiceTrait for AccountService {
    /// Creates a new account with currency exchange support.
    async fn create_account(&self, new_account: NewAccount) -> Result<Account> {
        let base_currency = self.base_currency.read().unwrap().clone();
        debug!(
            "Creating account..., base_currency: {}, new_account.currency: {}",
            base_currency, new_account.currency
        );

        // Perform async currency pair registration if needed
        if new_account.currency != base_currency {
            self.fx_service
                .register_currency_pair(new_account.currency.as_str(), base_currency.as_str())
                .await?;
        }

        // Repository handles transaction internally
        let result = self.repository.create(new_account).await?;

        // Emit AccountsChanged event with currency info for FX sync planning
        let currency_changes = vec![CurrencyChange {
            account_id: result.id.clone(),
            old_currency: None,
            new_currency: result.currency.clone(),
        }];
        self.event_sink.emit(DomainEvent::accounts_changed(
            vec![result.id.clone()],
            currency_changes,
        ));

        Ok(result)
    }

    /// Updates an existing account.
    async fn update_account(&self, account_update: AccountUpdate) -> Result<Account> {
        // Get existing account to detect changes
        let account_id = account_update.id.as_ref().ok_or_else(|| {
            crate::Error::Validation(crate::errors::ValidationError::InvalidInput(
                "Account ID is required".to_string(),
            ))
        })?;
        let existing = self.repository.get_by_id(account_id).await?;

        let mut account_update = account_update;

        // D-06: when account type transitions out of CREDIT_CARD, sanitize
        // the update payload to NULL all CC-only columns. Diesel's default
        // AsChangeset skips columns when the Option is None, so we must
        // actively set them to None here. The service-layer sanitization
        // guarantees the correct shape before validate() sees the payload.
        let type_transition_out_of_cc = existing.account_type
            == super::accounts_constants::account_types::CREDIT_CARD
            && account_update.account_type != super::accounts_constants::account_types::CREDIT_CARD;
        if type_transition_out_of_cc {
            account_update.credit_limit = None;
            account_update.statement_cycle_day = None;
            account_update.statement_balance = None;
            account_update.minimum_payment = None;
            account_update.statement_due_date = None;
            account_update.reward_points_balance = None;
            account_update.cashback_balance = None;
        }

        // D-12: auto-stamp balance_updated_at when current_balance changes.
        // The client never gets to set this field — server is the source of truth
        // for "when was the balance last touched". Defense-in-depth: the inbound
        // DTO already sets this to None per H-03 fix in apps/server/src/models.rs,
        // but core code is the contract for ALL callers (Tauri IPC, future MCP,
        // tests), so we re-assert here.
        if account_update.current_balance.is_some()
            && account_update.current_balance != existing.current_balance
        {
            account_update.balance_updated_at = Some(chrono::Utc::now().naive_utc());
        } else {
            // Belt and suspenders for D-12: discard any inbound value that
            // bypassed the DTO sanitation (e.g., a Tauri caller passing the
            // core type directly). Server is the SOLE writer of this field.
            account_update.balance_updated_at = None;
        }

        let result = self.repository.update(account_update).await?;

        // Detect currency changes and register FX pair if needed
        let currency_changes = if existing.currency != result.currency {
            let base_currency = self.base_currency.read().unwrap().clone();
            if result.currency != base_currency {
                self.fx_service
                    .register_currency_pair(result.currency.as_str(), base_currency.as_str())
                    .await?;
            }
            vec![CurrencyChange {
                account_id: result.id.clone(),
                old_currency: Some(existing.currency.clone()),
                new_currency: result.currency.clone(),
            }]
        } else {
            vec![]
        };

        // Emit AccountsChanged event
        self.event_sink.emit(DomainEvent::accounts_changed(
            vec![result.id.clone()],
            currency_changes,
        ));

        // Detect tracking mode changes
        if existing.tracking_mode != result.tracking_mode {
            let is_connected = result.provider_account_id.is_some();
            self.event_sink.emit(DomainEvent::tracking_mode_changed(
                result.id.clone(),
                existing.tracking_mode,
                result.tracking_mode,
                is_connected,
            ));
        }

        Ok(result)
    }

    /// Retrieves an account by its ID.
    async fn get_account(&self, account_id: &str) -> Result<Account> {
        self.repository.get_by_id(account_id).await
    }

    /// Lists all accounts with optional filtering by active status, archived status, and account IDs.
    async fn list_accounts(
        &self,
        is_active_filter: Option<bool>,
        is_archived_filter: Option<bool>,
        account_ids: Option<&[String]>,
    ) -> Result<Vec<Account>> {
        self.repository
            .list(is_active_filter, is_archived_filter, account_ids)
            .await
    }

    /// Lists all accounts.
    async fn get_all_accounts(&self) -> Result<Vec<Account>> {
        self.repository.list(None, None, None).await
    }

    /// Lists only active accounts.
    async fn get_active_accounts(&self) -> Result<Vec<Account>> {
        self.list_accounts(Some(true), None, None).await
    }

    /// Retrieves multiple accounts by their IDs.
    async fn get_accounts_by_ids(&self, account_ids: &[String]) -> Result<Vec<Account>> {
        self.list_accounts(None, None, Some(account_ids)).await
    }

    /// Returns all non-archived accounts (for aggregates/history)
    async fn get_non_archived_accounts(&self) -> Result<Vec<Account>> {
        self.repository.list(None, Some(false), None).await
    }

    /// Returns active, non-archived accounts (for UI selectors)
    async fn get_active_non_archived_accounts(&self) -> Result<Vec<Account>> {
        self.repository.list(Some(true), Some(false), None).await
    }

    fn get_base_currency(&self) -> Option<String> {
        let base_currency = self.base_currency.read().unwrap().clone();
        if base_currency.trim().is_empty() {
            None
        } else {
            Some(base_currency)
        }
    }

    /// Deletes an account by its ID.
    async fn delete_account(&self, account_id: &str) -> Result<()> {
        self.repository.delete(account_id).await?;

        // Clean up orphaned assets (activities are already CASCADE-deleted)
        match self
            .asset_repository
            .deactivate_orphaned_investments()
            .await
        {
            Ok(deactivated_ids) => {
                if !deactivated_ids.is_empty() {
                    info!(
                        "Deactivated {} orphaned investment assets after account deletion",
                        deactivated_ids.len()
                    );
                    for id in &deactivated_ids {
                        if let Err(e) = self.sync_state_store.delete(id).await {
                            warn!(
                                "Failed to delete sync state for orphaned asset {}: {}",
                                id, e
                            );
                        }
                    }
                }
            }
            Err(e) => {
                warn!(
                    "Failed to deactivate orphaned assets after account deletion: {}",
                    e
                );
            }
        }

        // Emit AccountsChanged event (no currency changes on delete)
        self.event_sink.emit(DomainEvent::accounts_changed(
            vec![account_id.to_string()],
            vec![],
        ));

        Ok(())
    }
}
