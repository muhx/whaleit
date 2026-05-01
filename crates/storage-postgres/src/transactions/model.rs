//! Diesel database models for transactions.

use chrono::{NaiveDate, NaiveDateTime};
use diesel::prelude::*;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use whaleit_core::transactions::{
    NewSplit, NewTransaction, PayeeCategoryMemory, Transaction, TransactionSplit, TransactionUpdate,
};

use crate::schema::{payee_category_memory, transaction_splits, transactions};

/// Diesel model for the `transactions` table.
#[derive(
    Debug,
    Clone,
    Queryable,
    Identifiable,
    Insertable,
    AsChangeset,
    Selectable,
    Serialize,
    Deserialize,
)]
#[diesel(table_name = transactions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TransactionDB {
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
}

/// Diesel model for the `transaction_splits` table.
#[derive(Debug, Clone, Queryable, Identifiable, Insertable, AsChangeset, Selectable)]
#[diesel(table_name = transaction_splits)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TransactionSplitDB {
    pub id: String,
    pub transaction_id: String,
    pub category_id: String,
    pub amount: Decimal,
    pub notes: Option<String>,
    pub sort_order: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// Diesel model for the `payee_category_memory` table.
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset, Selectable)]
#[diesel(table_name = payee_category_memory)]
#[diesel(primary_key(account_id, normalized_merchant))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PayeeCategoryMemoryDB {
    pub account_id: String,
    pub normalized_merchant: String,
    pub category_id: String,
    pub last_seen_at: NaiveDateTime,
    pub seen_count: i32,
}

// ── From<NewTransaction> for TransactionDB ──────────────────────────────────

impl From<NewTransaction> for TransactionDB {
    fn from(domain: NewTransaction) -> Self {
        let now = chrono::Utc::now().naive_utc();
        let has_splits = !domain.splits.is_empty();
        Self {
            id: Uuid::now_v7().to_string(),
            account_id: domain.account_id,
            direction: domain.direction,
            amount: domain.amount,
            currency: domain.currency,
            transaction_date: domain.transaction_date,
            payee: domain.payee,
            notes: domain.notes,
            category_id: domain.category_id,
            has_splits,
            fx_rate: domain.fx_rate,
            fx_rate_source: domain.fx_rate_source,
            transfer_group_id: domain.transfer_group_id,
            counterparty_account_id: domain.counterparty_account_id,
            transfer_leg_role: domain.transfer_leg_role,
            idempotency_key: domain.idempotency_key,
            import_run_id: domain.import_run_id,
            source: domain.source,
            external_ref: domain.external_ref,
            is_system_generated: domain.is_system_generated,
            is_user_modified: false,
            category_source: domain.category_source,
            created_at: now,
            updated_at: now,
        }
    }
}

// ── From<TransactionDB> for Transaction ─────────────────────────────────────

impl From<TransactionDB> for Transaction {
    fn from(db: TransactionDB) -> Self {
        Self {
            id: db.id,
            account_id: db.account_id,
            direction: db.direction,
            amount: db.amount,
            currency: db.currency,
            transaction_date: db.transaction_date,
            payee: db.payee,
            notes: db.notes,
            category_id: db.category_id,
            has_splits: db.has_splits,
            splits: vec![], // hydrated separately by repository
            fx_rate: db.fx_rate,
            fx_rate_source: db.fx_rate_source,
            transfer_group_id: db.transfer_group_id,
            counterparty_account_id: db.counterparty_account_id,
            transfer_leg_role: db.transfer_leg_role,
            idempotency_key: db.idempotency_key,
            import_run_id: db.import_run_id,
            source: db.source,
            external_ref: db.external_ref,
            is_system_generated: db.is_system_generated,
            is_user_modified: db.is_user_modified,
            category_source: db.category_source,
            created_at: db.created_at,
            updated_at: db.updated_at,
        }
    }
}

// ── Changeset struct for partial updates ────────────────────────────────────

/// Partial changeset for `diesel::update().set(...)` from `TransactionUpdate`.
/// Only `Some` fields are applied; `None` fields leave the column unchanged.
#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = transactions)]
#[diesel(treat_none_as_null = false)]
pub struct TransactionChangesetDB {
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
    pub is_user_modified: Option<bool>,
    pub category_source: Option<String>,
    pub updated_at: NaiveDateTime,
}

impl From<&TransactionUpdate> for TransactionChangesetDB {
    fn from(upd: &TransactionUpdate) -> Self {
        Self {
            direction: upd.direction.clone(),
            amount: upd.amount,
            currency: upd.currency.clone(),
            transaction_date: upd.transaction_date,
            payee: upd.payee.clone(),
            notes: upd.notes.clone(),
            category_id: upd.category_id.clone(),
            has_splits: upd.has_splits,
            fx_rate: upd.fx_rate,
            fx_rate_source: upd.fx_rate_source.clone(),
            transfer_group_id: upd.transfer_group_id.clone(),
            counterparty_account_id: upd.counterparty_account_id.clone(),
            transfer_leg_role: upd.transfer_leg_role.clone(),
            is_user_modified: upd.is_user_modified,
            category_source: upd.category_source.clone(),
            updated_at: chrono::Utc::now().naive_utc(),
        }
    }
}

// ── NewSplit → TransactionSplitDB ────────────────────────────────────────────

pub fn new_split_to_db(transaction_id: &str, split: &NewSplit) -> TransactionSplitDB {
    let now = chrono::Utc::now().naive_utc();
    TransactionSplitDB {
        id: Uuid::now_v7().to_string(),
        transaction_id: transaction_id.to_string(),
        category_id: split.category_id.clone(),
        amount: split.amount,
        notes: split.notes.clone(),
        sort_order: split.sort_order,
        created_at: now,
        updated_at: now,
    }
}

// ── From<TransactionSplitDB> for TransactionSplit ────────────────────────────

impl From<TransactionSplitDB> for TransactionSplit {
    fn from(db: TransactionSplitDB) -> Self {
        Self {
            id: db.id,
            transaction_id: db.transaction_id,
            category_id: db.category_id,
            amount: db.amount,
            notes: db.notes,
            sort_order: db.sort_order,
            created_at: db.created_at,
            updated_at: db.updated_at,
        }
    }
}

// ── PayeeCategoryMemory ↔ PayeeCategoryMemoryDB ──────────────────────────────

impl From<PayeeCategoryMemory> for PayeeCategoryMemoryDB {
    fn from(domain: PayeeCategoryMemory) -> Self {
        Self {
            account_id: domain.account_id,
            normalized_merchant: domain.normalized_merchant,
            category_id: domain.category_id,
            last_seen_at: domain.last_seen_at,
            seen_count: domain.seen_count,
        }
    }
}

impl From<PayeeCategoryMemoryDB> for PayeeCategoryMemory {
    fn from(db: PayeeCategoryMemoryDB) -> Self {
        Self {
            account_id: db.account_id,
            normalized_merchant: db.normalized_merchant,
            category_id: db.category_id,
            last_seen_at: db.last_seen_at,
            seen_count: db.seen_count,
        }
    }
}
