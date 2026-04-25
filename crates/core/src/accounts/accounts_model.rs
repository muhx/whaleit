//! Account domain models.

use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::accounts::account_types;
use crate::{errors::ValidationError, Error, Result};

/// Tracking mode for an account - determines how holdings are tracked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TrackingMode {
    /// Holdings are calculated from transaction history
    Transactions,
    /// Holdings are manually entered or imported directly
    Holdings,
    /// Tracking mode has not been set yet
    #[default]
    NotSet,
}

/// Domain model representing an account in the system.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub id: String,
    pub name: String,
    pub account_type: String,
    pub group: Option<String>,
    pub currency: String,
    pub is_default: bool,
    pub is_active: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub platform_id: Option<String>,
    /// Account number from the broker
    pub account_number: Option<String>,
    /// Additional metadata as JSON string
    pub meta: Option<String>,
    /// Provider name (e.g., 'SNAPTRADE', 'PLAID', 'MANUAL')
    pub provider: Option<String>,
    /// Account ID in the provider's system
    pub provider_account_id: Option<String>,
    /// Whether the account is archived
    pub is_archived: bool,
    /// Tracking mode for the account
    pub tracking_mode: TrackingMode,
    /// Free-text institution / issuer name (D-18). Distinct from platform_id.
    pub institution: Option<String>,
    /// Opening balance captured at account creation (D-11).
    pub opening_balance: Option<Decimal>,
    /// Latest manual current balance snapshot (D-12). Phase 4 will compute from txns.
    pub current_balance: Option<Decimal>,
    /// Timestamp of last current_balance update (D-12). Auto-stamped server-side.
    pub balance_updated_at: Option<NaiveDateTime>,
    /// Credit-card limit (CC-only, D-06).
    pub credit_limit: Option<Decimal>,
    /// Statement cycle day, 1..=31 (CC-only, D-06).
    pub statement_cycle_day: Option<i16>,
    /// Latest statement balance snapshot (CC-only, D-06, D-07).
    pub statement_balance: Option<Decimal>,
    /// Latest minimum payment due (CC-only, D-06).
    pub minimum_payment: Option<Decimal>,
    /// Latest statement due date (CC-only, D-06).
    pub statement_due_date: Option<NaiveDate>,
    /// Reward points balance (CC-only, D-09).
    pub reward_points_balance: Option<i32>,
    /// Cashback balance (CC-only, D-09).
    pub cashback_balance: Option<Decimal>,
}

/// Input model for creating a new account.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewAccount {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub name: String,
    pub account_type: String,
    pub group: Option<String>,
    pub currency: String,
    pub is_default: bool,
    pub is_active: bool,
    pub platform_id: Option<String>,
    pub account_number: Option<String>,
    pub meta: Option<String>,
    pub provider: Option<String>,
    pub provider_account_id: Option<String>,
    #[serde(default)]
    pub is_archived: bool,
    #[serde(default)]
    pub tracking_mode: TrackingMode,
    /// Free-text institution / issuer name (D-18). Distinct from platform_id.
    pub institution: Option<String>,
    /// Opening balance captured at account creation (D-11).
    pub opening_balance: Option<Decimal>,
    /// Latest manual current balance snapshot (D-12). Phase 4 will compute from txns.
    pub current_balance: Option<Decimal>,
    /// Timestamp of last current_balance update (D-12). Auto-stamped server-side.
    pub balance_updated_at: Option<NaiveDateTime>,
    /// Credit-card limit (CC-only, D-06).
    pub credit_limit: Option<Decimal>,
    /// Statement cycle day, 1..=31 (CC-only, D-06).
    pub statement_cycle_day: Option<i16>,
    /// Latest statement balance snapshot (CC-only, D-06, D-07).
    pub statement_balance: Option<Decimal>,
    /// Latest minimum payment due (CC-only, D-06).
    pub minimum_payment: Option<Decimal>,
    /// Latest statement due date (CC-only, D-06).
    pub statement_due_date: Option<NaiveDate>,
    /// Reward points balance (CC-only, D-09).
    pub reward_points_balance: Option<i32>,
    /// Cashback balance (CC-only, D-09).
    pub cashback_balance: Option<Decimal>,
}

impl NewAccount {
    /// Validates the new account data.
    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(Error::Validation(ValidationError::InvalidInput(
                "Account name cannot be empty".to_string(),
            )));
        }
        if self.currency.trim().is_empty() {
            return Err(Error::Validation(ValidationError::InvalidInput(
                "Currency cannot be empty".to_string(),
            )));
        }

        let is_credit_card = self.account_type == account_types::CREDIT_CARD;
        let is_bank_or_loan = matches!(
            self.account_type.as_str(),
            account_types::CHECKING | account_types::SAVINGS | account_types::LOAN
        );

        // D-06: CC-only fields must all be null for non-CC accounts.
        if !is_credit_card {
            if self.credit_limit.is_some()
                || self.statement_cycle_day.is_some()
                || self.statement_balance.is_some()
                || self.minimum_payment.is_some()
                || self.statement_due_date.is_some()
                || self.reward_points_balance.is_some()
                || self.cashback_balance.is_some()
            {
                return Err(Error::Validation(ValidationError::InvalidInput(
                    "Credit card fields are only valid for CREDIT_CARD accounts".to_string(),
                )));
            }
        } else {
            // CC required: credit_limit > 0 and statement_cycle_day in 1..=31.
            match self.credit_limit {
                Some(ref limit) if *limit > Decimal::ZERO => {}
                _ => {
                    return Err(Error::Validation(ValidationError::InvalidInput(
                        "Credit limit must be greater than 0".to_string(),
                    )));
                }
            }
            match self.statement_cycle_day {
                Some(d) if (1..=31).contains(&d) => {}
                _ => {
                    return Err(Error::Validation(ValidationError::InvalidInput(
                        "Statement cycle day must be between 1 and 31".to_string(),
                    )));
                }
            }
        }

        // D-11: opening_balance required for bank/CC/LOAN.
        if (is_bank_or_loan || is_credit_card) && self.opening_balance.is_none() {
            return Err(Error::Validation(ValidationError::InvalidInput(
                "Opening balance is required for bank, credit card, and loan accounts".to_string(),
            )));
        }

        // Bank/LOAN: opening_balance >= 0. CC: any value allowed (cards may
        // start with existing debt, stored positive per D-13).
        if is_bank_or_loan {
            if let Some(ref ob) = self.opening_balance {
                if *ob < Decimal::ZERO {
                    return Err(Error::Validation(ValidationError::InvalidInput(
                        "Opening balance cannot be negative for bank or loan accounts".to_string(),
                    )));
                }
            }
        }

        Ok(())
    }
}

/// Input model for updating an existing account.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountUpdate {
    pub id: Option<String>,
    pub name: String,
    pub account_type: String,
    pub group: Option<String>,
    pub is_default: bool,
    pub is_active: bool,
    pub platform_id: Option<String>,
    pub account_number: Option<String>,
    pub meta: Option<String>,
    pub provider: Option<String>,
    pub provider_account_id: Option<String>,
    pub is_archived: Option<bool>,
    pub tracking_mode: Option<TrackingMode>,
    /// Free-text institution / issuer name (D-18). Distinct from platform_id.
    pub institution: Option<String>,
    /// Opening balance captured at account creation (D-11).
    pub opening_balance: Option<Decimal>,
    /// Latest manual current balance snapshot (D-12). Phase 4 will compute from txns.
    pub current_balance: Option<Decimal>,
    /// Timestamp of last current_balance update (D-12). Auto-stamped server-side.
    pub balance_updated_at: Option<NaiveDateTime>,
    /// Credit-card limit (CC-only, D-06).
    pub credit_limit: Option<Decimal>,
    /// Statement cycle day, 1..=31 (CC-only, D-06).
    pub statement_cycle_day: Option<i16>,
    /// Latest statement balance snapshot (CC-only, D-06, D-07).
    pub statement_balance: Option<Decimal>,
    /// Latest minimum payment due (CC-only, D-06).
    pub minimum_payment: Option<Decimal>,
    /// Latest statement due date (CC-only, D-06).
    pub statement_due_date: Option<NaiveDate>,
    /// Reward points balance (CC-only, D-09).
    pub reward_points_balance: Option<i32>,
    /// Cashback balance (CC-only, D-09).
    pub cashback_balance: Option<Decimal>,
}

impl AccountUpdate {
    /// Validates the account update data.
    pub fn validate(&self) -> Result<()> {
        if self.id.is_none() {
            return Err(Error::Validation(ValidationError::InvalidInput(
                "Account ID is required for updates".to_string(),
            )));
        }
        if self.name.trim().is_empty() {
            return Err(Error::Validation(ValidationError::InvalidInput(
                "Account name cannot be empty".to_string(),
            )));
        }

        // D-06: same null-rule on updates.
        let is_credit_card = self.account_type == account_types::CREDIT_CARD;
        if !is_credit_card {
            if self.credit_limit.is_some()
                || self.statement_cycle_day.is_some()
                || self.statement_balance.is_some()
                || self.minimum_payment.is_some()
                || self.statement_due_date.is_some()
                || self.reward_points_balance.is_some()
                || self.cashback_balance.is_some()
            {
                return Err(Error::Validation(ValidationError::InvalidInput(
                    "Credit card fields are only valid for CREDIT_CARD accounts".to_string(),
                )));
            }
        }

        Ok(())
    }
}
