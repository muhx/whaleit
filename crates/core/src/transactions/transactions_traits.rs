//! Transaction repository, service, and supporting request/response types.

use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::Result;

use super::{
    duplicate_detector::DuplicateMatch,
    transactions_model::{NewTransaction, PayeeCategoryMemory, Transaction, TransactionUpdate},
};

// ---------------------------------------------------------------------------
// Supporting types
// ---------------------------------------------------------------------------

/// Controls how a paired-transfer amount edit propagates (D-04).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransferEditMode {
    /// Apply amount change to both legs (preserves pairing).
    ApplyBoth,
    /// Apply amount change to this leg only; breaks the transfer link.
    ThisLegOnly,
}

/// Input for one leg of a new transfer (D-01/D-02).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewTransferLeg {
    pub account_id: String,
    pub amount: Decimal,
    pub currency: String,
    pub transaction_date: NaiveDate,
    pub notes: Option<String>,
    pub category_id: Option<String>,
    pub fx_rate: Option<Decimal>,
    pub fx_rate_source: Option<String>,
}

/// Filter criteria for transaction search (TXN-03).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TransactionFilters {
    pub account_ids: Option<Vec<String>>,
    pub direction: Option<String>,
    pub date_from: Option<NaiveDate>,
    pub date_to: Option<NaiveDate>,
    pub payee_contains: Option<String>,
    pub category_ids: Option<Vec<String>>,
    pub show_transfers: Option<bool>,
    pub source: Option<String>,
    pub import_run_id: Option<String>,
}

/// Paginated transaction search result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionSearchResult {
    pub items: Vec<Transaction>,
    pub total: i64,
}

/// Transaction with running balance (for TXN-09 view).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionWithRunningBalance {
    pub txn: Transaction,
    pub running_balance: Decimal,
}

/// Request for a CSV import (TXN-04).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CsvImportRequest {
    pub account_id: String,
    pub account_currency: String,
    pub csv_bytes: Vec<u8>,
    pub mapping: super::csv_parser::CsvFieldMapping,
    pub import_run_id: Option<String>,
}

/// Request for an OFX import (TXN-05).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OfxImportRequest {
    pub account_id: String,
    pub account_currency: String,
    pub ofx_bytes: Vec<u8>,
    pub import_run_id: Option<String>,
}

/// Result of a CSV or OFX import operation.
///
/// `inserted_row_ids[i]` is the ID of the transaction inserted from input
/// row `i`. Skipped (duplicate) rows use an empty string sentinel `""`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub import_run_id: String,
    pub inserted: usize,
    pub skipped_duplicates: usize,
    pub errors: Vec<String>,
    /// Order-preserving: index i → ID of transaction from input row i.
    /// Empty string for rows that were skipped due to idempotency / duplicate.
    pub inserted_row_ids: Vec<String>,
    /// Duplicate matches found during import (for wizard Review step).
    pub duplicate_matches: Vec<DuplicateMatch>,
}

// ---------------------------------------------------------------------------
// Repository traits
// ---------------------------------------------------------------------------

/// Repository trait for transaction persistence (implemented by plan 04-03).
#[async_trait]
pub trait TransactionRepositoryTrait: Send + Sync {
    async fn create_with_splits(&self, new: NewTransaction) -> Result<Transaction>;
    async fn create_many_with_splits(&self, news: Vec<NewTransaction>) -> Result<Vec<Transaction>>;
    async fn update_with_splits(&self, update: TransactionUpdate) -> Result<Transaction>;
    async fn delete(&self, id: &str) -> Result<()>;
    async fn delete_pair(&self, transfer_group_id: &str) -> Result<()>;
    async fn get_by_id(&self, id: &str) -> Result<Transaction>;
    async fn get_by_idempotency_key(
        &self,
        account_id: &str,
        key: &str,
    ) -> Result<Option<Transaction>>;
    async fn search(
        &self,
        filters: TransactionFilters,
        page: i64,
        page_size: i64,
    ) -> Result<TransactionSearchResult>;
    async fn list_by_account_recent(
        &self,
        account_id: &str,
        limit: i64,
    ) -> Result<Vec<Transaction>>;
    async fn list_with_running_balance(
        &self,
        account_id: &str,
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
    ) -> Result<Vec<TransactionWithRunningBalance>>;
    async fn list_in_dup_window(
        &self,
        account_id: &str,
        date_lo: NaiveDate,
        date_hi: NaiveDate,
        amount_lo: Decimal,
        amount_hi: Decimal,
    ) -> Result<Vec<Transaction>>;
    async fn has_user_transactions(&self, account_id: &str) -> Result<bool>;
}

/// Repository trait for payee→category memory (D-12).
#[async_trait]
pub trait PayeeCategoryMemoryRepositoryTrait: Send + Sync {
    async fn lookup(
        &self,
        account_id: &str,
        normalized_merchant: &str,
    ) -> Result<Option<PayeeCategoryMemory>>;
    async fn list_for_account(&self, account_id: &str) -> Result<Vec<PayeeCategoryMemory>>;
    async fn upsert(&self, mem: PayeeCategoryMemory) -> Result<()>;
}

// ---------------------------------------------------------------------------
// Service trait
// ---------------------------------------------------------------------------

/// Service trait for transaction business logic.
#[async_trait]
pub trait TransactionServiceTrait: Send + Sync {
    async fn create_transaction(&self, new: NewTransaction) -> Result<Transaction>;
    async fn update_transaction(
        &self,
        update: TransactionUpdate,
        edit_mode: TransferEditMode,
    ) -> Result<Transaction>;
    async fn delete_transaction(&self, id: &str) -> Result<Transaction>;
    async fn get_transaction(&self, id: &str) -> Result<Transaction>;
    async fn search_transactions(
        &self,
        filters: TransactionFilters,
        page: i64,
        page_size: i64,
    ) -> Result<TransactionSearchResult>;
    async fn list_running_balance(
        &self,
        account_id: &str,
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
    ) -> Result<Vec<TransactionWithRunningBalance>>;
    async fn create_transfer(
        &self,
        src: NewTransferLeg,
        dst: NewTransferLeg,
    ) -> Result<(Transaction, Transaction)>;
    async fn break_transfer_pair(&self, leg_id: &str) -> Result<Transaction>;
    async fn detect_import_duplicates(
        &self,
        candidates: Vec<NewTransaction>,
    ) -> Result<Vec<DuplicateMatch>>;
    async fn import_csv(&self, req: CsvImportRequest) -> Result<ImportResult>;
    async fn import_ofx(&self, req: OfxImportRequest) -> Result<ImportResult>;
    async fn lookup_payee_category(
        &self,
        account_id: &str,
        payee: &str,
    ) -> Result<Option<PayeeCategoryMemory>>;
    async fn list_payee_category_memory(
        &self,
        account_id: &str,
    ) -> Result<Vec<PayeeCategoryMemory>>;
}
