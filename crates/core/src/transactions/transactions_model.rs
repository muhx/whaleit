//! Transaction domain models (Phase 4).

use chrono::{NaiveDate, NaiveDateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::{errors::ValidationError, Error, Result};

/// A persisted transaction row.
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
    /// Server-side hydrated splits (not persisted on the transactions row).
    pub splits: Vec<TransactionSplit>,
}

/// Input model for creating a new transaction.
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
    pub has_splits: bool,
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
    pub splits: Vec<NewSplit>,
}

impl NewTransaction {
    /// Validates the new transaction data.
    pub fn validate(&self) -> Result<()> {
        if self.amount <= Decimal::ZERO {
            return Err(Error::Validation(ValidationError::InvalidInput(
                "Amount must be greater than 0".to_string(),
            )));
        }

        if self.currency.trim().is_empty() {
            return Err(Error::Validation(ValidationError::InvalidInput(
                "Currency is required".to_string(),
            )));
        }

        let tomorrow = Utc::now()
            .date_naive()
            .succ_opt()
            .unwrap_or(Utc::now().date_naive());
        if self.transaction_date > tomorrow {
            return Err(Error::Validation(ValidationError::InvalidInput(
                "Date can't be more than a day in the future".to_string(),
            )));
        }

        use crate::transactions::transactions_constants::direction;
        let is_transfer = self.direction == direction::TRANSFER;

        if !is_transfer {
            let payee_empty = self
                .payee
                .as_deref()
                .map(str::trim)
                .unwrap_or("")
                .is_empty();
            if payee_empty {
                return Err(Error::Validation(ValidationError::InvalidInput(
                    "Payee is required".to_string(),
                )));
            }
        }

        if is_transfer {
            if self.transfer_group_id.is_none()
                || self.counterparty_account_id.is_none()
                || self.transfer_leg_role.is_none()
            {
                return Err(Error::Validation(ValidationError::InvalidInput(
                    "Transfer requires group, counterparty, and leg role".to_string(),
                )));
            }
        } else if self.transfer_group_id.is_some() {
            return Err(Error::Validation(ValidationError::InvalidInput(
                "transfer_group_id only valid on transfer rows".to_string(),
            )));
        }

        if self.has_splits {
            if self.splits.is_empty() {
                return Err(Error::Validation(ValidationError::InvalidInput(
                    "Split totals must equal the transaction amount".to_string(),
                )));
            }
            let sum: Decimal = self.splits.iter().map(|s| s.amount).sum();
            if (sum - self.amount).abs() > Decimal::new(1, 2) {
                return Err(Error::Validation(ValidationError::InvalidInput(
                    "Split totals must equal the transaction amount".to_string(),
                )));
            }
        }

        match (self.fx_rate.is_some(), self.fx_rate_source.is_some()) {
            (true, false) | (false, true) => {
                return Err(Error::Validation(ValidationError::InvalidInput(
                    "fx_rate and fx_rate_source must both be set or both unset".to_string(),
                )));
            }
            _ => {}
        }

        Ok(())
    }
}

/// Input model for updating an existing transaction.
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
    pub fx_rate: Option<Decimal>,
    pub fx_rate_source: Option<String>,
    pub transfer_group_id: Option<String>,
    pub counterparty_account_id: Option<String>,
    pub transfer_leg_role: Option<String>,
    pub idempotency_key: Option<String>,
    pub import_run_id: Option<String>,
    pub source: Option<String>,
    pub external_ref: Option<String>,
    pub is_system_generated: Option<bool>,
    pub is_user_modified: Option<bool>,
    pub category_source: Option<String>,
    /// Replacement semantics: when `Some`, ALL existing splits are deleted
    /// and the supplied set is inserted (no per-split partial updates).
    /// When `None`, existing splits are left untouched.
    pub splits: Option<Vec<NewSplit>>,
}

impl TransactionUpdate {
    /// Validates the transaction update data.
    pub fn validate(&self) -> Result<()> {
        if self.id.trim().is_empty() {
            return Err(Error::Validation(ValidationError::InvalidInput(
                "Transaction ID is required for updates".to_string(),
            )));
        }

        if let Some(ref amt) = self.amount {
            if *amt <= Decimal::ZERO {
                return Err(Error::Validation(ValidationError::InvalidInput(
                    "Amount must be greater than 0".to_string(),
                )));
            }
        }

        if let Some(ref cur) = self.currency {
            if cur.trim().is_empty() {
                return Err(Error::Validation(ValidationError::InvalidInput(
                    "Currency is required".to_string(),
                )));
            }
        }

        match (self.fx_rate.is_some(), self.fx_rate_source.is_some()) {
            (true, false) | (false, true) => {
                return Err(Error::Validation(ValidationError::InvalidInput(
                    "fx_rate and fx_rate_source must both be set or both unset".to_string(),
                )));
            }
            _ => {}
        }

        Ok(())
    }
}

/// A split row attached to a transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionSplit {
    pub id: String,
    pub transaction_id: String,
    /// Splits MUST have a category (TXN-08) — DB column is NOT NULL.
    pub category_id: String,
    pub amount: Decimal,
    pub notes: Option<String>,
    pub sort_order: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// Input for creating a new split.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewSplit {
    pub category_id: String,
    pub amount: Decimal,
    pub notes: Option<String>,
    pub sort_order: i32,
}

/// Input for updating an existing split.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SplitUpdate {
    pub id: String,
    pub category_id: Option<String>,
    pub amount: Option<Decimal>,
    pub notes: Option<String>,
    pub sort_order: Option<i32>,
}

/// Merchant→category learning row (D-12).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PayeeCategoryMemory {
    pub account_id: String,
    pub normalized_merchant: String,
    pub category_id: String,
    pub last_seen_at: NaiveDateTime,
    pub seen_count: i32,
}
