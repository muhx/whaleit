//! Transaction repository and service traits.

use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::Decimal;

use super::transactions_model::{
    NewTransaction, PayeeCategoryMemory, Transaction, TransactionFilters, TransactionSearchResult,
    TransactionUpdate, TransactionWithRunningBalance,
};
use crate::errors::Result;

/// Repository trait for transaction CRUD and query operations.
#[async_trait]
pub trait TransactionRepositoryTrait: Send + Sync {
    /// Creates a transaction and its splits atomically.
    async fn create_with_splits(&self, new: NewTransaction) -> Result<Transaction>;

    /// Creates multiple transactions and their splits atomically.
    /// Returns inserted transactions in the SAME ORDER as the input slice.
    async fn create_many_with_splits(&self, news: Vec<NewTransaction>) -> Result<Vec<Transaction>>;

    /// Updates a transaction, replacing its splits atomically.
    async fn update_with_splits(&self, update: TransactionUpdate) -> Result<Transaction>;

    /// Deletes a single transaction (splits cascade via FK ON DELETE CASCADE).
    async fn delete(&self, id: &str) -> Result<()>;

    /// Deletes both legs of a transfer pair by transfer_group_id.
    async fn delete_pair(&self, transfer_group_id: &str) -> Result<()>;

    /// Gets a transaction by ID, hydrating its splits.
    async fn get_by_id(&self, id: &str) -> Result<Transaction>;

    /// Looks up a transaction by its idempotency key (for re-import dedup).
    async fn get_by_idempotency_key(
        &self,
        account_id: &str,
        key: &str,
    ) -> Result<Option<Transaction>>;

    /// Searches transactions with filters, pagination.
    async fn search(
        &self,
        filters: TransactionFilters,
        page: i64,
        page_size: i64,
    ) -> Result<TransactionSearchResult>;

    /// Lists recent transactions for an account.
    async fn list_by_account_recent(
        &self,
        account_id: &str,
        limit: i64,
    ) -> Result<Vec<Transaction>>;

    /// Queries the running-balance VIEW for an account with optional date range.
    async fn list_with_running_balance(
        &self,
        account_id: &str,
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
    ) -> Result<Vec<TransactionWithRunningBalance>>;

    /// Returns transactions within a date+amount window (for duplicate detection).
    async fn list_in_dup_window(
        &self,
        account_id: &str,
        date_lo: NaiveDate,
        date_hi: NaiveDate,
        amount_lo: Decimal,
        amount_hi: Decimal,
    ) -> Result<Vec<Transaction>>;

    /// Returns true if the account has at least one user-entered transaction.
    async fn has_user_transactions(&self, account_id: &str) -> Result<bool>;
}

/// Repository trait for payee → category memory (D-12).
#[async_trait]
pub trait PayeeCategoryMemoryRepositoryTrait: Send + Sync {
    /// Looks up category memory for a (account_id, normalized_merchant) pair.
    async fn lookup(
        &self,
        account_id: &str,
        normalized_merchant: &str,
    ) -> Result<Option<PayeeCategoryMemory>>;

    /// Lists all memory entries for an account, ordered by last_seen_at DESC.
    async fn list_for_account(&self, account_id: &str) -> Result<Vec<PayeeCategoryMemory>>;

    /// Upserts a memory entry. Increments seen_count on conflict.
    async fn upsert(&self, mem: PayeeCategoryMemory) -> Result<()>;
}
