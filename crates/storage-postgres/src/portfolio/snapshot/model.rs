//! Database model for account state snapshots (PostgreSQL).

use diesel::prelude::*;
use rust_decimal::Decimal;
use std::str::FromStr;

use whaleit_core::constants::DECIMAL_PRECISION;
use whaleit_core::portfolio::snapshot::{AccountStateSnapshot, SnapshotSource};

/// Database read model for holdings_snapshots.
#[derive(
    Debug,
    Clone,
    Queryable,
    QueryableByName,
    Selectable,
    Identifiable,
    Insertable,
    AsChangeset,
    PartialEq,
)]
#[diesel(table_name = crate::schema::holdings_snapshots)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AccountStateSnapshotDB {
    pub id: String,
    pub account_id: String,
    pub snapshot_date: chrono::NaiveDate,
    pub currency: String,
    pub positions: String,
    pub cash_balances: String,
    pub cost_basis: String,
    pub net_contribution: String,
    pub calculated_at: chrono::NaiveDateTime,
    pub net_contribution_base: String,
    pub cash_total_account_currency: String,
    pub cash_total_base_currency: String,
    pub source: String,
}

impl From<AccountStateSnapshotDB> for AccountStateSnapshot {
    fn from(db: AccountStateSnapshotDB) -> Self {
        Self {
            id: db.id,
            account_id: db.account_id,
            snapshot_date: db.snapshot_date,
            currency: db.currency,
            positions: serde_json::from_str(&db.positions).unwrap_or_default(),
            cash_balances: serde_json::from_str(&db.cash_balances).unwrap_or_default(),
            cost_basis: Decimal::from_str(&db.cost_basis).unwrap_or_default(),
            net_contribution: Decimal::from_str(&db.net_contribution).unwrap_or_default(),
            net_contribution_base: Decimal::from_str(&db.net_contribution_base).unwrap_or_default(),
            cash_total_account_currency: Decimal::from_str(&db.cash_total_account_currency)
                .unwrap_or_default(),
            cash_total_base_currency: Decimal::from_str(&db.cash_total_base_currency)
                .unwrap_or_default(),
            calculated_at: db.calculated_at,
            source: serde_json::from_str(&format!("\"{}\"", db.source))
                .unwrap_or(SnapshotSource::Calculated),
        }
    }
}

impl From<AccountStateSnapshot> for AccountStateSnapshotDB {
    fn from(domain: AccountStateSnapshot) -> Self {
        Self {
            id: domain.id,
            account_id: domain.account_id,
            snapshot_date: domain.snapshot_date,
            currency: domain.currency,
            positions: serde_json::to_string(&domain.positions)
                .unwrap_or_else(|_| "{}".to_string()),
            cash_balances: serde_json::to_string(&domain.cash_balances)
                .unwrap_or_else(|_| "{}".to_string()),
            cost_basis: domain.cost_basis.round_dp(DECIMAL_PRECISION).to_string(),
            net_contribution: domain
                .net_contribution
                .round_dp(DECIMAL_PRECISION)
                .to_string(),
            net_contribution_base: domain
                .net_contribution_base
                .round_dp(DECIMAL_PRECISION)
                .to_string(),
            cash_total_account_currency: domain
                .cash_total_account_currency
                .round_dp(DECIMAL_PRECISION)
                .to_string(),
            cash_total_base_currency: domain
                .cash_total_base_currency
                .round_dp(DECIMAL_PRECISION)
                .to_string(),
            calculated_at: domain.calculated_at,
            source: serde_json::to_string(&domain.source)
                .unwrap_or_else(|_| "\"CALCULATED\"".to_string())
                .trim_matches('"')
                .to_string(),
        }
    }
}
