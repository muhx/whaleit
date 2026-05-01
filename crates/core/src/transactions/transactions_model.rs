//! Transaction domain models.

use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Direction of a transaction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransactionDirection {
    Income,
    Expense,
    Transfer,
}

impl TransactionDirection {
    pub fn as_str(&self) -> &'static str {
        match self {
            TransactionDirection::Income => "INCOME",
            TransactionDirection::Expense => "EXPENSE",
            TransactionDirection::Transfer => "TRANSFER",
        }
    }
}

impl From<&str> for TransactionDirection {
    fn from(s: &str) -> Self {
        match s {
            "INCOME" => TransactionDirection::Income,
            "EXPENSE" => TransactionDirection::Expense,
            _ => TransactionDirection::Transfer,
        }
    }
}

/// A category split across a parent transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionSplit {
    pub id: String,
    pub transaction_id: String,
    pub category_id: String,
    pub amount: Decimal,
    pub notes: Option<String>,
    pub sort_order: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// Input for creating a transaction split.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewSplit {
    pub category_id: String,
    pub amount: Decimal,
    pub notes: Option<String>,
    pub sort_order: i32,
}

/// Full transaction domain model.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub id: String,
    pub account_id: String,
    pub direction: String,
    pub amount: Decimal,
    pub currency: String,
    pub transaction_date: NaiveDate,
    pub payee: Option<String>,
    pub notes: Option<String>,
    pub category_id: Option<String>,
    pub has_splits: bool,
    pub splits: Vec<TransactionSplit>,
    pub fx_rate: Option<Decimal>,
    pub fx_rate_source: Option<String>,
    pub transfer_group_id: Option<String>,
    pub counterparty_account_id: Option<String>,
    pub transfer_leg_role: Option<String>,
    pub idempotency_key: Option<String>,
    pub import_run_id: Option<String>,
    pub source: String,
    pub external_ref: Option<String>,
    pub is_system_generated: bool,
    pub is_user_modified: bool,
    pub category_source: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// Input for creating a new transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewTransaction {
    pub account_id: String,
    pub direction: String,
    pub amount: Decimal,
    pub currency: String,
    pub transaction_date: NaiveDate,
    pub payee: Option<String>,
    pub notes: Option<String>,
    pub category_id: Option<String>,
    pub splits: Vec<NewSplit>,
    pub fx_rate: Option<Decimal>,
    pub fx_rate_source: Option<String>,
    pub transfer_group_id: Option<String>,
    pub counterparty_account_id: Option<String>,
    pub transfer_leg_role: Option<String>,
    pub idempotency_key: Option<String>,
    pub import_run_id: Option<String>,
    pub source: String,
    pub external_ref: Option<String>,
    pub is_system_generated: bool,
    pub category_source: Option<String>,
}

/// Partial update input for a transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionUpdate {
    pub id: String,
    pub direction: Option<String>,
    pub amount: Option<Decimal>,
    pub currency: Option<String>,
    pub transaction_date: Option<NaiveDate>,
    pub payee: Option<String>,
    pub notes: Option<String>,
    pub category_id: Option<String>,
    pub has_splits: Option<bool>,
    pub splits: Option<Vec<NewSplit>>,
    pub fx_rate: Option<Decimal>,
    pub fx_rate_source: Option<String>,
    pub transfer_group_id: Option<String>,
    pub counterparty_account_id: Option<String>,
    pub transfer_leg_role: Option<String>,
    pub is_user_modified: Option<bool>,
    pub category_source: Option<String>,
}

/// Transaction with running balance (from the VIEW).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionWithRunningBalance {
    pub txn: Transaction,
    pub running_balance: Decimal,
}

/// Payee → category memory entry (D-12).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PayeeCategoryMemory {
    pub account_id: String,
    pub normalized_merchant: String,
    pub category_id: String,
    pub last_seen_at: NaiveDateTime,
    pub seen_count: i32,
}

/// Filters for transaction search.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionFilters {
    pub account_ids: Vec<String>,
    pub category_ids: Vec<String>,
    pub directions: Vec<String>,
    pub amount_min: Option<Decimal>,
    pub amount_max: Option<Decimal>,
    pub date_from: Option<NaiveDate>,
    pub date_to: Option<NaiveDate>,
    pub show_transfers: bool,
    pub search_keyword: Option<String>,
}

/// Paginated search result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionSearchResult {
    pub items: Vec<Transaction>,
    pub total: i64,
}

/// Result of a batch import operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub inserted: Vec<Transaction>,
    pub skipped_duplicate_keys: Vec<String>,
}
