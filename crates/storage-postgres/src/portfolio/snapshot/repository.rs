//! PostgreSQL snapshot repository implementation.

use async_trait::async_trait;
use chrono::NaiveDate;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use log::{debug, warn};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use super::model::AccountStateSnapshotDB;
use crate::db::PgPool;
use crate::errors::StoragePgError;
use crate::schema::accounts::dsl as accounts_dsl;
use crate::schema::holdings_snapshots::dsl as hs_dsl;
use whaleit_core::constants::PORTFOLIO_TOTAL_ACCOUNT_ID;
use whaleit_core::errors::Result;
use whaleit_core::portfolio::snapshot::{AccountStateSnapshot, SnapshotRepositoryTrait};

/// Source constant for calculated snapshots — preserve MANUAL/BROKER/CSV on overwrite.
const SOURCE_CALCULATED: &str = "CALCULATED";

pub struct PgSnapshotRepository {
    pool: Arc<PgPool>,
}

impl PgSnapshotRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SnapshotRepositoryTrait for PgSnapshotRepository {
    async fn save_snapshots(&self, snapshots: &[AccountStateSnapshot]) -> Result<()> {
        if snapshots.is_empty() {
            return Ok(());
        }

        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;

        let db_models: Vec<AccountStateSnapshotDB> = snapshots
            .iter()
            .cloned()
            .map(AccountStateSnapshotDB::from)
            .collect();

        // PostgreSQL uses ON CONFLICT instead of replace_into
        // Note: EXCLUDED.* column names are raw SQL strings — update if schema columns change.
        // snapshot_date is intentionally excluded as dates are immutable per ID.
        for chunk in db_models.chunks(100) {
            diesel::insert_into(hs_dsl::holdings_snapshots)
                .values(chunk)
                .on_conflict(hs_dsl::id)
                .do_update()
                .set((
                    hs_dsl::currency.eq(diesel::dsl::sql("EXCLUDED.currency")),
                    hs_dsl::positions.eq(diesel::dsl::sql("EXCLUDED.positions")),
                    hs_dsl::cash_balances.eq(diesel::dsl::sql("EXCLUDED.cash_balances")),
                    hs_dsl::cost_basis.eq(diesel::dsl::sql("EXCLUDED.cost_basis")),
                    hs_dsl::net_contribution.eq(diesel::dsl::sql("EXCLUDED.net_contribution")),
                    hs_dsl::calculated_at.eq(diesel::dsl::sql("EXCLUDED.calculated_at")),
                    hs_dsl::net_contribution_base.eq(diesel::dsl::sql("EXCLUDED.net_contribution_base")),
                    hs_dsl::cash_total_account_currency.eq(diesel::dsl::sql("EXCLUDED.cash_total_account_currency")),
                    hs_dsl::cash_total_base_currency.eq(diesel::dsl::sql("EXCLUDED.cash_total_base_currency")),
                    hs_dsl::source.eq(diesel::dsl::sql("EXCLUDED.source")),
                ))
                .execute(&mut conn)
                .await
                .map_err(StoragePgError::from)?;
        }

        Ok(())
    }

    async fn get_snapshots_by_account(
        &self,
        account_id: &str,
        start_date: Option<NaiveDate>,
        end_date: Option<NaiveDate>,
    ) -> Result<Vec<AccountStateSnapshot>> {
        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;

        let mut query = hs_dsl::holdings_snapshots
            .filter(hs_dsl::account_id.eq(account_id))
            .into_boxed();

        if let Some(start) = start_date {
            query = query.filter(hs_dsl::snapshot_date.ge(start));
        }
        if let Some(end) = end_date {
            query = query.filter(hs_dsl::snapshot_date.le(end));
        }

        let results = query
            .order(hs_dsl::snapshot_date.asc())
            .select(AccountStateSnapshotDB::as_select())
            .load::<AccountStateSnapshotDB>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        Ok(results.into_iter().map(AccountStateSnapshot::from).collect())
    }

    async fn get_latest_snapshot_before_date(
        &self,
        account_id: &str,
        date: NaiveDate,
    ) -> Result<Option<AccountStateSnapshot>> {
        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;

        let result = hs_dsl::holdings_snapshots
            .filter(hs_dsl::account_id.eq(account_id))
            .filter(hs_dsl::snapshot_date.le(date))
            .order(hs_dsl::snapshot_date.desc())
            .select(AccountStateSnapshotDB::as_select())
            .first::<AccountStateSnapshotDB>(&mut conn)
            .await
            .optional()
            .map_err(StoragePgError::from)?;

        Ok(result.map(AccountStateSnapshot::from))
    }

    async fn get_latest_snapshots_before_date(
        &self,
        account_ids: &[String],
        date: NaiveDate,
    ) -> Result<HashMap<String, AccountStateSnapshot>> {
        if account_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;

        // PG DISTINCT ON for latest-per-account
        let sql = format!(
            "SELECT DISTINCT ON (account_id) \
             id, account_id, snapshot_date, currency, positions, \
             cash_balances, cost_basis, net_contribution, calculated_at, net_contribution_base, \
             cash_total_account_currency, cash_total_base_currency, source \
             FROM holdings_snapshots \
             WHERE account_id = ANY($1) AND snapshot_date <= $2 \
             ORDER BY account_id, snapshot_date DESC"
        );

        let results: Vec<AccountStateSnapshotDB> = diesel::sql_query(sql)
            .bind::<diesel::sql_types::Array<diesel::sql_types::Text>, Vec<String>>(
                account_ids.to_vec(),
            )
            .bind::<diesel::sql_types::Date, NaiveDate>(date)
            .load::<AccountStateSnapshotDB>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        Ok(results
            .into_iter()
            .map(|db| {
                let acc_id = db.account_id.clone();
                (acc_id, AccountStateSnapshot::from(db))
            })
            .collect())
    }

    async fn get_all_latest_snapshots(
        &self,
        account_ids: &[String],
    ) -> Result<HashMap<String, AccountStateSnapshot>> {
        if account_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;

        let sql = format!(
            "SELECT DISTINCT ON (account_id) \
             id, account_id, snapshot_date, currency, positions, \
             cash_balances, cost_basis, net_contribution, calculated_at, net_contribution_base, \
             cash_total_account_currency, cash_total_base_currency, source \
             FROM holdings_snapshots \
             WHERE account_id = ANY($1) \
             ORDER BY account_id, snapshot_date DESC"
        );

        let results: Vec<AccountStateSnapshotDB> = diesel::sql_query(sql)
            .bind::<diesel::sql_types::Array<diesel::sql_types::Text>, Vec<String>>(
                account_ids.to_vec(),
            )
            .load::<AccountStateSnapshotDB>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        Ok(results
            .into_iter()
            .map(|db| {
                let acc_id = db.account_id.clone();
                (acc_id, AccountStateSnapshot::from(db))
            })
            .collect())
    }

    async fn delete_snapshots_by_account_ids(&self, account_ids: &[String]) -> Result<usize> {
        if account_ids.is_empty() {
            return Ok(0);
        }

        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;

        // Only delete CALCULATED snapshots — preserve manual/imported
        let count = diesel::delete(
            hs_dsl::holdings_snapshots
                .filter(hs_dsl::account_id.eq_any(account_ids))
                .filter(hs_dsl::source.eq(SOURCE_CALCULATED)),
        )
        .execute(&mut conn)
        .await
        .map_err(StoragePgError::from)?;

        Ok(count)
    }

    async fn delete_snapshots_for_account_and_dates(
        &self,
        account_id: &str,
        dates_to_delete: &[NaiveDate],
    ) -> Result<()> {
        if dates_to_delete.is_empty() {
            return Ok(());
        }

        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;

        diesel::delete(
            hs_dsl::holdings_snapshots
                .filter(hs_dsl::account_id.eq(account_id))
                .filter(hs_dsl::snapshot_date.eq_any(dates_to_delete)),
        )
        .execute(&mut conn)
        .await
        .map_err(StoragePgError::from)?;

        Ok(())
    }

    async fn delete_snapshots_for_account_in_range(
        &self,
        account_id: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<()> {
        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;

        diesel::delete(
            hs_dsl::holdings_snapshots
                .filter(hs_dsl::account_id.eq(account_id))
                .filter(hs_dsl::snapshot_date.ge(start_date))
                .filter(hs_dsl::snapshot_date.le(end_date)),
        )
        .execute(&mut conn)
        .await
        .map_err(StoragePgError::from)?;

        Ok(())
    }

    async fn overwrite_snapshots_for_account_in_range(
        &self,
        target_account_id: &str,
        range_start_date: NaiveDate,
        range_end_date: NaiveDate,
        snapshots_to_save: &[AccountStateSnapshot],
    ) -> Result<()> {
        // Delete only CALCULATED snapshots in range
        self.delete_calculated_snapshots_for_account_in_range(
            target_account_id,
            range_start_date,
            range_end_date,
        )
        .await?;

        // Get anchor (non-calculated) dates to avoid overwriting
        let anchor_dates = self
            .get_anchor_snapshot_dates_for_account_in_range(
                target_account_id,
                range_start_date,
                range_end_date,
            )
            .await?;

        if !snapshots_to_save.is_empty() {
            let mut filtered: Vec<AccountStateSnapshot> = snapshots_to_save
                .iter()
                .filter(|s| s.account_id == target_account_id)
                .cloned()
                .collect();

            if !anchor_dates.is_empty() {
                filtered.retain(|s| !anchor_dates.contains(&s.snapshot_date));
            }

            if !filtered.is_empty() {
                self.save_snapshots(&filtered).await?;
            }
        }

        Ok(())
    }

    async fn overwrite_multiple_account_snapshot_ranges(
        &self,
        new_snapshots: &[AccountStateSnapshot],
    ) -> Result<()> {
        if new_snapshots.is_empty() {
            return Ok(());
        }

        let mut by_account: HashMap<String, Vec<AccountStateSnapshot>> = HashMap::new();
        for snap in new_snapshots {
            by_account
                .entry(snap.account_id.clone())
                .or_default()
                .push(snap.clone());
        }

        for (acc_id, acc_snaps) in by_account {
            if acc_snaps.is_empty() {
                continue;
            }

            let min_date = acc_snaps.iter().map(|s| s.snapshot_date).min().unwrap();
            let max_date = acc_snaps.iter().map(|s| s.snapshot_date).max().unwrap();

            self.overwrite_snapshots_for_account_in_range(&acc_id, min_date, max_date, &acc_snaps)
                .await?;
        }

        Ok(())
    }

    async fn get_total_portfolio_snapshots(
        &self,
        start_date: Option<NaiveDate>,
        end_date: Option<NaiveDate>,
    ) -> Result<Vec<AccountStateSnapshot>> {
        self.get_snapshots_by_account(PORTFOLIO_TOTAL_ACCOUNT_ID, start_date, end_date)
            .await
    }

    async fn get_all_non_archived_account_snapshots(
        &self,
        start_date: Option<NaiveDate>,
        end_date: Option<NaiveDate>,
    ) -> Result<Vec<AccountStateSnapshot>> {
        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;

        // Get non-archived account IDs
        let non_archived_ids: Vec<String> = accounts_dsl::accounts
            .filter(accounts_dsl::is_archived.eq(false))
            .select(accounts_dsl::id)
            .load::<String>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        if non_archived_ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut query = hs_dsl::holdings_snapshots
            .filter(hs_dsl::account_id.ne("TOTAL"))
            .filter(hs_dsl::account_id.eq_any(&non_archived_ids))
            .into_boxed();

        if let Some(start) = start_date {
            query = query.filter(hs_dsl::snapshot_date.ge(start));
        }
        if let Some(end) = end_date {
            query = query.filter(hs_dsl::snapshot_date.le(end));
        }

        let results = query
            .order(hs_dsl::snapshot_date.asc())
            .select(AccountStateSnapshotDB::as_select())
            .load::<AccountStateSnapshotDB>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        Ok(results.into_iter().map(AccountStateSnapshot::from).collect())
    }

    async fn get_earliest_snapshot_date(&self, account_id: &str) -> Result<Option<NaiveDate>> {
        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;

        let result = hs_dsl::holdings_snapshots
            .filter(hs_dsl::account_id.eq(account_id))
            .select(hs_dsl::snapshot_date)
            .order(hs_dsl::snapshot_date.asc())
            .first::<NaiveDate>(&mut conn)
            .await
            .optional()
            .map_err(StoragePgError::from)?;

        Ok(result)
    }

    async fn overwrite_all_snapshots_for_account(
        &self,
        target_account_id: &str,
        snapshots_to_save: &[AccountStateSnapshot],
    ) -> Result<()> {
        let anchor_dates = self
            .get_anchor_snapshot_dates_for_account(target_account_id)
            .await?;

        let filtered: Vec<AccountStateSnapshot> = if anchor_dates.is_empty() {
            snapshots_to_save.to_vec()
        } else {
            snapshots_to_save
                .iter()
                .filter(|s| !anchor_dates.contains(&s.snapshot_date))
                .cloned()
                .collect()
        };

        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;

        // Delete only CALCULATED snapshots
        diesel::delete(
            hs_dsl::holdings_snapshots
                .filter(hs_dsl::account_id.eq(target_account_id))
                .filter(hs_dsl::source.eq(SOURCE_CALCULATED)),
        )
        .execute(&mut conn)
        .await
        .map_err(StoragePgError::from)?;

        // Save new ones
        if !filtered.is_empty() {
            let db_models: Vec<AccountStateSnapshotDB> = filtered
                .into_iter()
                .map(AccountStateSnapshotDB::from)
                .collect();

            // Note: EXCLUDED.* column names are raw SQL strings — update if schema columns change.
            for chunk in db_models.chunks(100) {
                diesel::insert_into(hs_dsl::holdings_snapshots)
                    .values(chunk)
                    .on_conflict(hs_dsl::id)
                    .do_update()
                    .set((
                        hs_dsl::currency.eq(diesel::dsl::sql("EXCLUDED.currency")),
                        hs_dsl::positions.eq(diesel::dsl::sql("EXCLUDED.positions")),
                        hs_dsl::cash_balances.eq(diesel::dsl::sql("EXCLUDED.cash_balances")),
                        hs_dsl::cost_basis.eq(diesel::dsl::sql("EXCLUDED.cost_basis")),
                        hs_dsl::net_contribution.eq(diesel::dsl::sql("EXCLUDED.net_contribution")),
                        hs_dsl::calculated_at.eq(diesel::dsl::sql("EXCLUDED.calculated_at")),
                        hs_dsl::net_contribution_base.eq(diesel::dsl::sql("EXCLUDED.net_contribution_base")),
                        hs_dsl::cash_total_account_currency.eq(diesel::dsl::sql("EXCLUDED.cash_total_account_currency")),
                        hs_dsl::cash_total_base_currency.eq(diesel::dsl::sql("EXCLUDED.cash_total_base_currency")),
                        hs_dsl::source.eq(diesel::dsl::sql("EXCLUDED.source")),
                    ))
                    .execute(&mut conn)
                    .await
                    .map_err(StoragePgError::from)?;
            }
        }

        Ok(())
    }

    async fn update_snapshots_source(&self, account_id: &str, new_source: &str) -> Result<usize> {
        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;

        let count = diesel::update(hs_dsl::holdings_snapshots.filter(hs_dsl::account_id.eq(account_id)))
            .set(hs_dsl::source.eq(new_source))
            .execute(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        Ok(count)
    }

    async fn save_or_update_snapshot(&self, snapshot: &AccountStateSnapshot) -> Result<()> {
        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;

        let db_model = AccountStateSnapshotDB::from(snapshot.clone());

        // Note: EXCLUDED.* column names are raw SQL strings — update if schema columns change.
        diesel::insert_into(hs_dsl::holdings_snapshots)
            .values(&db_model)
            .on_conflict(hs_dsl::id)
            .do_update()
            .set((
                hs_dsl::currency.eq(diesel::dsl::sql("EXCLUDED.currency")),
                hs_dsl::positions.eq(diesel::dsl::sql("EXCLUDED.positions")),
                hs_dsl::cash_balances.eq(diesel::dsl::sql("EXCLUDED.cash_balances")),
                hs_dsl::cost_basis.eq(diesel::dsl::sql("EXCLUDED.cost_basis")),
                hs_dsl::net_contribution.eq(diesel::dsl::sql("EXCLUDED.net_contribution")),
                hs_dsl::calculated_at.eq(diesel::dsl::sql("EXCLUDED.calculated_at")),
                hs_dsl::net_contribution_base.eq(diesel::dsl::sql("EXCLUDED.net_contribution_base")),
                hs_dsl::cash_total_account_currency.eq(diesel::dsl::sql("EXCLUDED.cash_total_account_currency")),
                hs_dsl::cash_total_base_currency.eq(diesel::dsl::sql("EXCLUDED.cash_total_base_currency")),
                hs_dsl::source.eq(diesel::dsl::sql("EXCLUDED.source")),
            ))
            .execute(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        Ok(())
    }

    async fn get_non_calculated_snapshot_count(&self, account_id: &str) -> Result<usize> {
        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;

        let count: i64 = hs_dsl::holdings_snapshots
            .filter(hs_dsl::account_id.eq(account_id))
            .filter(hs_dsl::source.ne(SOURCE_CALCULATED))
            .count()
            .get_result(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        Ok(count as usize)
    }

    async fn get_earliest_non_calculated_snapshot(
        &self,
        account_id: &str,
    ) -> Result<Option<AccountStateSnapshot>> {
        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;

        let result = hs_dsl::holdings_snapshots
            .filter(hs_dsl::account_id.eq(account_id))
            .filter(hs_dsl::source.ne(SOURCE_CALCULATED))
            .order(hs_dsl::snapshot_date.asc())
            .select(AccountStateSnapshotDB::as_select())
            .first::<AccountStateSnapshotDB>(&mut conn)
            .await
            .optional()
            .map_err(StoragePgError::from)?;

        Ok(result.map(AccountStateSnapshot::from))
    }
}

// =============================================================================
// Private helper methods
// =============================================================================

impl PgSnapshotRepository {
    /// Delete only CALCULATED snapshots for account in a date range.
    async fn delete_calculated_snapshots_for_account_in_range(
        &self,
        account_id: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<usize> {
        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;

        let count = diesel::delete(
            hs_dsl::holdings_snapshots
                .filter(hs_dsl::account_id.eq(account_id))
                .filter(hs_dsl::snapshot_date.ge(start_date))
                .filter(hs_dsl::snapshot_date.le(end_date))
                .filter(hs_dsl::source.eq(SOURCE_CALCULATED)),
        )
        .execute(&mut conn)
        .await
        .map_err(StoragePgError::from)?;

        Ok(count)
    }

    /// Get dates of non-calculated snapshots in a range (anchors to preserve during overwrite).
    async fn get_anchor_snapshot_dates_for_account_in_range(
        &self,
        account_id: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<HashSet<NaiveDate>> {
        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;

        let dates: Vec<NaiveDate> = hs_dsl::holdings_snapshots
            .select(hs_dsl::snapshot_date)
            .filter(hs_dsl::account_id.eq(account_id))
            .filter(hs_dsl::snapshot_date.ge(start_date))
            .filter(hs_dsl::snapshot_date.le(end_date))
            .filter(hs_dsl::source.ne(SOURCE_CALCULATED))
            .load::<NaiveDate>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        Ok(dates.into_iter().collect())
    }

    /// Get all non-calculated snapshot dates for an account (anchors to preserve during overwrite).
    async fn get_anchor_snapshot_dates_for_account(
        &self,
        account_id: &str,
    ) -> Result<HashSet<NaiveDate>> {
        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;

        let dates: Vec<NaiveDate> = hs_dsl::holdings_snapshots
            .select(hs_dsl::snapshot_date)
            .filter(hs_dsl::account_id.eq(account_id))
            .filter(hs_dsl::source.ne(SOURCE_CALCULATED))
            .load::<NaiveDate>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        Ok(dates.into_iter().collect())
    }
}
