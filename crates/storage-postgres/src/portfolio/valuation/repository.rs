//! PostgreSQL valuation repository implementation.

use async_trait::async_trait;
use chrono::NaiveDate;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use super::model::DailyAccountValuationDB;
use crate::db::PgPool;
use crate::errors::StoragePgError;
use crate::schema::daily_account_valuation::dsl as dav_dsl;
use whaleit_core::errors::Result;
use whaleit_core::portfolio::valuation::{
    DailyAccountValuation, NegativeBalanceInfo, ValuationRepositoryTrait,
};

pub struct PgValuationRepository {
    pool: Arc<PgPool>,
}

impl PgValuationRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ValuationRepositoryTrait for PgValuationRepository {
    async fn save_valuations(&self, valuation_records: &[DailyAccountValuation]) -> Result<()> {
        if valuation_records.is_empty() {
            return Ok(());
        }

        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;

        let db_records: Vec<DailyAccountValuationDB> = valuation_records
            .iter()
            .cloned()
            .map(DailyAccountValuationDB::from)
            .collect();

        // Chunk to avoid oversized queries
        // Note: EXCLUDED.* column names are raw SQL strings — update if schema columns change.
        for chunk in db_records.chunks(1000) {
            diesel::insert_into(dav_dsl::daily_account_valuation)
                .values(chunk)
                .on_conflict(dav_dsl::id)
                .do_update()
                .set((
                    dav_dsl::account_currency.eq(diesel::dsl::sql("EXCLUDED.account_currency")),
                    dav_dsl::base_currency.eq(diesel::dsl::sql("EXCLUDED.base_currency")),
                    dav_dsl::fx_rate_to_base.eq(diesel::dsl::sql("EXCLUDED.fx_rate_to_base")),
                    dav_dsl::cash_balance.eq(diesel::dsl::sql("EXCLUDED.cash_balance")),
                    dav_dsl::investment_market_value
                        .eq(diesel::dsl::sql("EXCLUDED.investment_market_value")),
                    dav_dsl::total_value.eq(diesel::dsl::sql("EXCLUDED.total_value")),
                    dav_dsl::cost_basis.eq(diesel::dsl::sql("EXCLUDED.cost_basis")),
                    dav_dsl::net_contribution.eq(diesel::dsl::sql("EXCLUDED.net_contribution")),
                    dav_dsl::calculated_at.eq(diesel::dsl::sql("EXCLUDED.calculated_at")),
                ))
                .execute(&mut conn)
                .await
                .map_err(StoragePgError::from)?;
        }

        Ok(())
    }

    async fn get_historical_valuations(
        &self,
        account_id: &str,
        start_date: Option<NaiveDate>,
        end_date: Option<NaiveDate>,
    ) -> Result<Vec<DailyAccountValuation>> {
        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;

        let mut query = dav_dsl::daily_account_valuation
            .filter(dav_dsl::account_id.eq(account_id))
            .order(dav_dsl::valuation_date.asc())
            .into_boxed();

        if let Some(start) = start_date {
            query = query.filter(dav_dsl::valuation_date.ge(start));
        }
        if let Some(end) = end_date {
            query = query.filter(dav_dsl::valuation_date.le(end));
        }

        let results = query
            .select(DailyAccountValuationDB::as_select())
            .load::<DailyAccountValuationDB>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        Ok(results
            .into_iter()
            .map(DailyAccountValuation::from)
            .collect())
    }

    async fn load_latest_valuation_date(&self, account_id: &str) -> Result<Option<NaiveDate>> {
        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;

        let result: Option<Option<NaiveDate>> = dav_dsl::daily_account_valuation
            .filter(dav_dsl::account_id.eq(account_id))
            .select(diesel::dsl::max(dav_dsl::valuation_date))
            .first::<Option<NaiveDate>>(&mut conn)
            .await
            .optional()
            .map_err(StoragePgError::from)?;

        Ok(result.flatten())
    }

    async fn delete_valuations_for_account(
        &self,
        account_id: &str,
        since_date: Option<NaiveDate>,
    ) -> Result<()> {
        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;

        let mut query = diesel::delete(dav_dsl::daily_account_valuation)
            .filter(dav_dsl::account_id.eq(account_id))
            .into_boxed();

        if let Some(date) = since_date {
            query = query.filter(dav_dsl::valuation_date.ge(date));
        }

        query
            .execute(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        Ok(())
    }

    async fn get_latest_valuations(
        &self,
        account_ids: &[String],
    ) -> Result<Vec<DailyAccountValuation>> {
        if account_ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;

        // PG DISTINCT ON for latest-per-account
        let sql = format!(
            "SELECT DISTINCT ON (account_id) \
             id, account_id, valuation_date, account_currency, base_currency, \
             fx_rate_to_base, cash_balance, investment_market_value, total_value, \
             cost_basis, net_contribution, calculated_at \
             FROM daily_account_valuation \
             WHERE account_id = ANY($1) \
             ORDER BY account_id, valuation_date DESC"
        );

        let results: Vec<DailyAccountValuationDB> = diesel::sql_query(sql)
            .bind::<diesel::sql_types::Array<diesel::sql_types::Text>, Vec<String>>(
                account_ids.to_vec(),
            )
            .load::<DailyAccountValuationDB>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        // Maintain input order
        let mut results_map: HashMap<String, DailyAccountValuation> = results
            .into_iter()
            .map(|db| {
                let acc_id = db.account_id.clone();
                (acc_id, DailyAccountValuation::from(db))
            })
            .collect();

        let mut ordered = Vec::with_capacity(account_ids.len());
        for acc_id in account_ids {
            if let Some(val) = results_map.remove(acc_id) {
                ordered.push(val);
            }
        }

        Ok(ordered)
    }

    async fn get_valuations_on_date(
        &self,
        account_ids: &[String],
        date: NaiveDate,
    ) -> Result<Vec<DailyAccountValuation>> {
        if account_ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;

        let results = dav_dsl::daily_account_valuation
            .filter(dav_dsl::account_id.eq_any(account_ids))
            .filter(dav_dsl::valuation_date.eq(date))
            .select(DailyAccountValuationDB::as_select())
            .load::<DailyAccountValuationDB>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        Ok(results
            .into_iter()
            .map(DailyAccountValuation::from)
            .collect())
    }

    async fn get_accounts_with_negative_balance(
        &self,
        account_ids: &[String],
    ) -> Result<Vec<NegativeBalanceInfo>> {
        if account_ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;

        // PG: use DISTINCT ON to get the earliest negative per account
        // Guard against non-numeric total_value with regex before casting
        let sql = format!(
            "SELECT DISTINCT ON (account_id) \
             account_id, valuation_date AS first_negative_date, \
             cash_balance, total_value, account_currency \
             FROM daily_account_valuation \
             WHERE account_id = ANY($1) \
             AND total_value ~ '^-?\\d' AND total_value::numeric < 0 \
             ORDER BY account_id, valuation_date ASC"
        );

        #[derive(QueryableByName, Debug)]
        struct NegativeBalanceRow {
            #[diesel(sql_type = diesel::sql_types::Text)]
            account_id: String,
            #[diesel(sql_type = diesel::sql_types::Date)]
            first_negative_date: NaiveDate,
            #[diesel(sql_type = diesel::sql_types::Text)]
            cash_balance: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            total_value: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            account_currency: String,
        }

        let rows: Vec<NegativeBalanceRow> = diesel::sql_query(sql)
            .bind::<diesel::sql_types::Array<diesel::sql_types::Text>, Vec<String>>(
                account_ids.to_vec(),
            )
            .load::<NegativeBalanceRow>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        let result = rows
            .into_iter()
            .filter_map(|r| {
                let cash = rust_decimal::Decimal::from_str(&r.cash_balance).ok()?;
                let total = rust_decimal::Decimal::from_str(&r.total_value).ok()?;
                Some(NegativeBalanceInfo {
                    account_id: r.account_id,
                    first_negative_date: r.first_negative_date,
                    cash_balance: cash,
                    total_value: total,
                    account_currency: r.account_currency,
                })
            })
            .collect();

        Ok(result)
    }
}
