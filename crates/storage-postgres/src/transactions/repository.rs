//! PostgreSQL transaction repository implementation.

use std::sync::Arc;

use async_trait::async_trait;
use chrono::NaiveDate;
use diesel::prelude::*;
use diesel::sql_types::{Nullable, Numeric, Text};
use diesel_async::scoped_futures::ScopedFutureExt;
use diesel_async::{AsyncConnection, RunQueryDsl};
use rust_decimal::Decimal;

use whaleit_core::errors::{DatabaseError, Result};
use whaleit_core::transactions::{
    NewTransaction, PayeeCategoryMemory, PayeeCategoryMemoryRepositoryTrait, Transaction,
    TransactionFilters, TransactionRepositoryTrait, TransactionSearchResult, TransactionSplit,
    TransactionUpdate, TransactionWithRunningBalance,
};

use crate::db::{PgConnection, PgPool};
use crate::errors::{IntoCore, StoragePgError};
use crate::schema::{payee_category_memory, transaction_splits, transactions};

use super::model::{
    new_split_to_db, PayeeCategoryMemoryDB, TransactionChangesetDB, TransactionDB,
    TransactionSplitDB,
};

pub struct PgTransactionRepository {
    pub(crate) pool: Arc<PgPool>,
}

impl PgTransactionRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

// ── Private helper functions ─────────────────────────────────────────────────

async fn hydrate_one(conn: &mut PgConnection<'_>, parent: TransactionDB) -> Result<Transaction> {
    let splits: Vec<TransactionSplitDB> = transaction_splits::table
        .filter(transaction_splits::transaction_id.eq(&parent.id))
        .order(transaction_splits::sort_order.asc())
        .load::<TransactionSplitDB>(conn)
        .await
        .into_core()?;
    let mut txn = Transaction::from(parent);
    txn.splits = splits.into_iter().map(TransactionSplit::from).collect();
    Ok(txn)
}

async fn hydrate_many(
    conn: &mut PgConnection<'_>,
    parents: Vec<TransactionDB>,
) -> Result<Vec<Transaction>> {
    if parents.is_empty() {
        return Ok(vec![]);
    }
    let ids: Vec<String> = parents.iter().map(|p| p.id.clone()).collect();
    let all_splits: Vec<TransactionSplitDB> = transaction_splits::table
        .filter(transaction_splits::transaction_id.eq_any(&ids))
        .order(transaction_splits::sort_order.asc())
        .load::<TransactionSplitDB>(conn)
        .await
        .into_core()?;

    Ok(parents
        .into_iter()
        .map(|parent| {
            let splits: Vec<TransactionSplit> = all_splits
                .iter()
                .filter(|s| s.transaction_id == parent.id)
                .cloned()
                .map(TransactionSplit::from)
                .collect();
            let mut txn = Transaction::from(parent);
            txn.splits = splits;
            txn
        })
        .collect())
}

// ── Queryable struct for v_transactions_with_running_balance VIEW ─────────────

#[derive(QueryableByName, Debug)]
struct RunningBalanceRow {
    #[diesel(sql_type = Text)]
    id: String,
    #[diesel(sql_type = Text)]
    account_id: String,
    #[diesel(sql_type = Text)]
    direction: String,
    #[diesel(sql_type = Numeric)]
    amount: Decimal,
    #[diesel(sql_type = Text)]
    currency: String,
    #[diesel(sql_type = diesel::sql_types::Date)]
    transaction_date: NaiveDate,
    #[diesel(sql_type = Nullable<Text>)]
    payee: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    notes: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    category_id: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    has_splits: bool,
    #[diesel(sql_type = Nullable<Numeric>)]
    fx_rate: Option<Decimal>,
    #[diesel(sql_type = Nullable<Text>)]
    fx_rate_source: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    transfer_group_id: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    counterparty_account_id: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    transfer_leg_role: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    idempotency_key: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    import_run_id: Option<String>,
    #[diesel(sql_type = Text)]
    source: String,
    #[diesel(sql_type = Nullable<Text>)]
    external_ref: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    is_system_generated: bool,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    is_user_modified: bool,
    #[diesel(sql_type = Nullable<Text>)]
    category_source: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Timestamp)]
    created_at: chrono::NaiveDateTime,
    #[diesel(sql_type = diesel::sql_types::Timestamp)]
    updated_at: chrono::NaiveDateTime,
    #[diesel(sql_type = Numeric)]
    running_balance: Decimal,
}

impl From<RunningBalanceRow> for TransactionWithRunningBalance {
    fn from(row: RunningBalanceRow) -> Self {
        let txn = Transaction {
            id: row.id,
            account_id: row.account_id,
            direction: row.direction,
            amount: row.amount,
            currency: row.currency,
            transaction_date: row.transaction_date,
            payee: row.payee,
            notes: row.notes,
            category_id: row.category_id,
            has_splits: row.has_splits,
            splits: vec![],
            fx_rate: row.fx_rate,
            fx_rate_source: row.fx_rate_source,
            transfer_group_id: row.transfer_group_id,
            counterparty_account_id: row.counterparty_account_id,
            transfer_leg_role: row.transfer_leg_role,
            idempotency_key: row.idempotency_key,
            import_run_id: row.import_run_id,
            source: row.source,
            external_ref: row.external_ref,
            is_system_generated: row.is_system_generated,
            is_user_modified: row.is_user_modified,
            category_source: row.category_source,
            created_at: row.created_at,
            updated_at: row.updated_at,
        };
        Self {
            txn,
            running_balance: row.running_balance,
        }
    }
}

// ── TransactionRepositoryTrait ────────────────────────────────────────────────

#[async_trait]
impl TransactionRepositoryTrait for PgTransactionRepository {
    async fn create_with_splits(&self, new: NewTransaction) -> Result<Transaction> {
        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;
        let parent_db: TransactionDB = new.clone().into();
        let parent_id = parent_db.id.clone();
        let splits_input = new.splits.clone();

        conn.transaction::<_, diesel::result::Error, _>(|conn| {
            let parent_db = parent_db.clone();
            let splits_input = splits_input.clone();
            let parent_id = parent_id.clone();
            async move {
                diesel::insert_into(transactions::table)
                    .values(&parent_db)
                    .execute(conn)
                    .await?;
                if !splits_input.is_empty() {
                    let split_dbs: Vec<TransactionSplitDB> = splits_input
                        .iter()
                        .map(|s| new_split_to_db(&parent_id, s))
                        .collect();
                    diesel::insert_into(transaction_splits::table)
                        .values(&split_dbs)
                        .execute(conn)
                        .await?;
                }
                Ok(())
            }
            .scope_boxed()
        })
        .await
        .into_core()?;

        self.get_by_id(&parent_id).await
    }

    async fn create_many_with_splits(&self, news: Vec<NewTransaction>) -> Result<Vec<Transaction>> {
        if news.is_empty() {
            return Ok(vec![]);
        }
        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;

        // Build DB rows preserving input order. Each gets a stable ID upfront.
        let parent_dbs: Vec<TransactionDB> = news.iter().map(|n| n.clone().into()).collect();
        let parent_ids: Vec<String> = parent_dbs.iter().map(|p| p.id.clone()).collect();

        let splits_by_txn: Vec<Vec<TransactionSplitDB>> = parent_dbs
            .iter()
            .zip(news.iter())
            .map(|(db, new)| {
                new.splits
                    .iter()
                    .map(|s| new_split_to_db(&db.id, s))
                    .collect()
            })
            .collect();

        conn.transaction::<_, diesel::result::Error, _>(|conn| {
            let parent_dbs = parent_dbs.clone();
            let splits_by_txn = splits_by_txn.clone();
            async move {
                for parent_db in &parent_dbs {
                    diesel::insert_into(transactions::table)
                        .values(parent_db)
                        .execute(conn)
                        .await?;
                }
                for split_dbs in &splits_by_txn {
                    if !split_dbs.is_empty() {
                        diesel::insert_into(transaction_splits::table)
                            .values(split_dbs)
                            .execute(conn)
                            .await?;
                    }
                }
                Ok(())
            }
            .scope_boxed()
        })
        .await
        .map_err(|e| {
            // Map idempotency_key unique violation to a named error.
            if let diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                ref info,
            ) = e
            {
                let msg = info.message().to_string();
                if msg.contains("idempotency_key") {
                    return whaleit_core::errors::Error::Database(DatabaseError::UniqueViolation(
                        msg,
                    ));
                }
            }
            StoragePgError::from(e).into()
        })?;

        // Re-fetch all inserted rows in INPUT ORDER.
        let rows: Vec<TransactionDB> = transactions::table
            .filter(transactions::id.eq_any(&parent_ids))
            .load::<TransactionDB>(&mut conn)
            .await
            .into_core()?;

        // Restore input order: parent_ids[i] -> rows[i].
        let mut ordered: Vec<Option<TransactionDB>> = vec![None; parent_ids.len()];
        for row in rows {
            if let Some(pos) = parent_ids.iter().position(|id| id == &row.id) {
                ordered[pos] = Some(row);
            }
        }
        let ordered: Vec<TransactionDB> = ordered.into_iter().flatten().collect();
        hydrate_many(&mut conn, ordered).await
    }

    async fn update_with_splits(&self, update: TransactionUpdate) -> Result<Transaction> {
        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;
        let txn_id = update.id.clone();
        let changeset = TransactionChangesetDB::from(&update);
        let should_replace = update.has_splits == Some(true) && update.splits.is_some();
        let new_splits = update.splits.clone().unwrap_or_default();

        conn.transaction::<_, diesel::result::Error, _>(|conn| {
            let txn_id = txn_id.clone();
            let changeset = changeset.clone();
            let new_splits = new_splits.clone();
            async move {
                diesel::update(transactions::table.find(&txn_id))
                    .set(&changeset)
                    .execute(conn)
                    .await?;
                if should_replace {
                    diesel::delete(
                        transaction_splits::table
                            .filter(transaction_splits::transaction_id.eq(&txn_id)),
                    )
                    .execute(conn)
                    .await?;
                    if !new_splits.is_empty() {
                        let split_dbs: Vec<TransactionSplitDB> = new_splits
                            .iter()
                            .map(|s| new_split_to_db(&txn_id, s))
                            .collect();
                        diesel::insert_into(transaction_splits::table)
                            .values(&split_dbs)
                            .execute(conn)
                            .await?;
                    }
                }
                Ok(())
            }
            .scope_boxed()
        })
        .await
        .into_core()?;

        self.get_by_id(&txn_id).await
    }

    async fn delete(&self, id: &str) -> Result<()> {
        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;
        diesel::delete(transactions::table.find(id))
            .execute(&mut conn)
            .await
            .into_core()?;
        Ok(())
    }

    async fn delete_pair(&self, transfer_group_id_val: &str) -> Result<()> {
        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;
        diesel::delete(
            transactions::table.filter(transactions::transfer_group_id.eq(transfer_group_id_val)),
        )
        .execute(&mut conn)
        .await
        .into_core()?;
        Ok(())
    }

    async fn get_by_id(&self, id: &str) -> Result<Transaction> {
        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;
        let parent: TransactionDB = transactions::table
            .find(id)
            .first::<TransactionDB>(&mut conn)
            .await
            .into_core()?;
        hydrate_one(&mut conn, parent).await
    }

    async fn get_by_idempotency_key(
        &self,
        account_id: &str,
        key: &str,
    ) -> Result<Option<Transaction>> {
        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;
        let res: Option<TransactionDB> = transactions::table
            .filter(transactions::account_id.eq(account_id))
            .filter(transactions::idempotency_key.eq(key))
            .first::<TransactionDB>(&mut conn)
            .await
            .optional()
            .into_core()?;
        // Splits not hydrated — lookup is for dedup, not display.
        Ok(res.map(Transaction::from))
    }

    async fn search(
        &self,
        filters: TransactionFilters,
        page: i64,
        page_size: i64,
    ) -> Result<TransactionSearchResult> {
        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;
        let offset = page * page_size;

        let mut query = transactions::table.into_boxed();
        let mut count_q = transactions::table.into_boxed();

        if !filters.account_ids.is_empty() {
            query = query.filter(transactions::account_id.eq_any(&filters.account_ids));
            count_q = count_q.filter(transactions::account_id.eq_any(&filters.account_ids));
        }
        if !filters.category_ids.is_empty() {
            query = query.filter(transactions::category_id.eq_any(&filters.category_ids));
            count_q = count_q.filter(transactions::category_id.eq_any(&filters.category_ids));
        }
        if !filters.directions.is_empty() {
            query = query.filter(transactions::direction.eq_any(&filters.directions));
            count_q = count_q.filter(transactions::direction.eq_any(&filters.directions));
        }
        if let Some(min) = filters.amount_min {
            query = query.filter(transactions::amount.ge(min));
            count_q = count_q.filter(transactions::amount.ge(min));
        }
        if let Some(max) = filters.amount_max {
            query = query.filter(transactions::amount.le(max));
            count_q = count_q.filter(transactions::amount.le(max));
        }
        if let Some(from) = filters.date_from {
            query = query.filter(transactions::transaction_date.ge(from));
            count_q = count_q.filter(transactions::transaction_date.ge(from));
        }
        if let Some(to) = filters.date_to {
            query = query.filter(transactions::transaction_date.le(to));
            count_q = count_q.filter(transactions::transaction_date.le(to));
        }
        if !filters.show_transfers {
            query = query.filter(transactions::direction.ne("TRANSFER"));
            count_q = count_q.filter(transactions::direction.ne("TRANSFER"));
        }
        if let Some(ref kw) = filters.search_keyword {
            let pat = format!("%{kw}%");
            query = query.filter(
                transactions::payee
                    .ilike(pat.clone())
                    .or(transactions::notes.ilike(pat.clone())),
            );
            count_q = count_q.filter(
                transactions::payee
                    .ilike(pat.clone())
                    .or(transactions::notes.ilike(pat)),
            );
        }

        let total: i64 = count_q.count().get_result(&mut conn).await.into_core()?;

        let rows: Vec<TransactionDB> = query
            .order((
                transactions::transaction_date.desc(),
                transactions::created_at.desc(),
            ))
            .limit(page_size)
            .offset(offset)
            .load::<TransactionDB>(&mut conn)
            .await
            .into_core()?;

        let items = hydrate_many(&mut conn, rows).await?;
        Ok(TransactionSearchResult { items, total })
    }

    async fn list_by_account_recent(
        &self,
        account_id: &str,
        limit: i64,
    ) -> Result<Vec<Transaction>> {
        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;
        let rows: Vec<TransactionDB> = transactions::table
            .filter(transactions::account_id.eq(account_id))
            .order((
                transactions::transaction_date.desc(),
                transactions::created_at.desc(),
            ))
            .limit(limit)
            .load::<TransactionDB>(&mut conn)
            .await
            .into_core()?;
        hydrate_many(&mut conn, rows).await
    }

    async fn list_with_running_balance(
        &self,
        account_id: &str,
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
    ) -> Result<Vec<TransactionWithRunningBalance>> {
        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;
        let rows: Vec<RunningBalanceRow> = diesel::sql_query(
            "SELECT * FROM v_transactions_with_running_balance \
             WHERE account_id = $1 \
             AND ($2::date IS NULL OR transaction_date >= $2) \
             AND ($3::date IS NULL OR transaction_date <= $3) \
             ORDER BY transaction_date DESC, created_at DESC",
        )
        .bind::<Text, _>(account_id)
        .bind::<Nullable<diesel::sql_types::Date>, _>(from)
        .bind::<Nullable<diesel::sql_types::Date>, _>(to)
        .load::<RunningBalanceRow>(&mut conn)
        .await
        .into_core()?;
        Ok(rows
            .into_iter()
            .map(TransactionWithRunningBalance::from)
            .collect())
    }

    async fn list_in_dup_window(
        &self,
        account_id: &str,
        date_lo: NaiveDate,
        date_hi: NaiveDate,
        amount_lo: Decimal,
        amount_hi: Decimal,
    ) -> Result<Vec<Transaction>> {
        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;
        // Single batched query per RESEARCH §Performance: 1000-Row Import.
        // Splits NOT hydrated — detector reads only top-level fields.
        let rows: Vec<TransactionDB> = transactions::table
            .filter(transactions::account_id.eq(account_id))
            .filter(transactions::transaction_date.between(date_lo, date_hi))
            .filter(transactions::amount.between(amount_lo, amount_hi))
            .order(transactions::transaction_date.asc())
            .load::<TransactionDB>(&mut conn)
            .await
            .into_core()?;
        Ok(rows.into_iter().map(Transaction::from).collect())
    }

    async fn has_user_transactions(&self, account_id: &str) -> Result<bool> {
        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;
        let count: i64 = transactions::table
            .filter(transactions::account_id.eq(account_id))
            .filter(transactions::is_system_generated.eq(false))
            .count()
            .get_result(&mut conn)
            .await
            .into_core()?;
        Ok(count > 0)
    }
}

// ── PayeeCategoryMemoryRepositoryTrait ───────────────────────────────────────

#[async_trait]
impl PayeeCategoryMemoryRepositoryTrait for PgTransactionRepository {
    async fn lookup(
        &self,
        account_id: &str,
        normalized_merchant: &str,
    ) -> Result<Option<PayeeCategoryMemory>> {
        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;
        let res: Option<PayeeCategoryMemoryDB> = payee_category_memory::table
            .filter(payee_category_memory::account_id.eq(account_id))
            .filter(payee_category_memory::normalized_merchant.eq(normalized_merchant))
            .first::<PayeeCategoryMemoryDB>(&mut conn)
            .await
            .optional()
            .into_core()?;
        Ok(res.map(PayeeCategoryMemory::from))
    }

    async fn list_for_account(&self, account_id: &str) -> Result<Vec<PayeeCategoryMemory>> {
        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;
        let rows: Vec<PayeeCategoryMemoryDB> = payee_category_memory::table
            .filter(payee_category_memory::account_id.eq(account_id))
            .order(payee_category_memory::last_seen_at.desc())
            .load::<PayeeCategoryMemoryDB>(&mut conn)
            .await
            .into_core()?;
        Ok(rows.into_iter().map(PayeeCategoryMemory::from).collect())
    }

    async fn upsert(&self, mem: PayeeCategoryMemory) -> Result<()> {
        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;
        let db = PayeeCategoryMemoryDB::from(mem);

        // ON CONFLICT (account_id, normalized_merchant) DO UPDATE
        diesel::sql_query(
            "INSERT INTO payee_category_memory \
             (account_id, normalized_merchant, category_id, last_seen_at, seen_count) \
             VALUES ($1, $2, $3, $4, $5) \
             ON CONFLICT (account_id, normalized_merchant) \
             DO UPDATE SET \
               category_id = EXCLUDED.category_id, \
               last_seen_at = EXCLUDED.last_seen_at, \
               seen_count = payee_category_memory.seen_count + 1",
        )
        .bind::<Text, _>(&db.account_id)
        .bind::<Text, _>(&db.normalized_merchant)
        .bind::<Text, _>(&db.category_id)
        .bind::<diesel::sql_types::Timestamp, _>(db.last_seen_at)
        .bind::<diesel::sql_types::Integer, _>(db.seen_count)
        .execute(&mut conn)
        .await
        .into_core()?;
        Ok(())
    }
}
