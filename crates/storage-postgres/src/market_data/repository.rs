//! PostgreSQL market data repository (QuoteStore + ProviderSettingsStore).

use async_trait::async_trait;
use chrono::NaiveDate;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use std::collections::HashMap;
use std::sync::Arc;

use super::model::{MarketDataProviderSettingDB, QuoteDB, UpdateMarketDataProviderSettingDB};
use crate::db::PgPool;
use crate::errors::StoragePgError;
use crate::schema::market_data_providers::dsl as mdp_dsl;
use crate::schema::quotes::dsl as q_dsl;
use whaleit_core::errors::{DatabaseError, Result};
use whaleit_core::quotes::{
    LatestQuotePair, MarketDataProviderSetting, ProviderSettingsStore, Quote, QuoteStore,
    UpdateMarketDataProviderSetting,
};
use whaleit_core::quotes::types::{AssetId, Day, QuoteSource};

pub struct PgMarketDataRepository {
    pool: Arc<PgPool>,
}

impl PgMarketDataRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

/// Source priority for tie-breaking latest-quote lookups on the same day.
const SOURCE_PRIORITY_CASE: &str =
    "CASE source WHEN 'MANUAL' THEN 1 WHEN 'BROKER' THEN 2 ELSE 3 END";

// =============================================================================
// QuoteStore Implementation
// =============================================================================

#[async_trait]
impl QuoteStore for PgMarketDataRepository {
    // =========================================================================
    // Mutations
    // =========================================================================

    async fn save_quote(&self, quote: &Quote) -> Result<Quote> {
        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;

        let quote_cloned = quote.clone();
        let day_str = quote_cloned.timestamp.format("%Y-%m-%d").to_string();
        let now = chrono::Utc::now().naive_utc();

        // Check existing
        let existing: Option<QuoteDB> = q_dsl::quotes
            .filter(q_dsl::asset_id.eq(&quote_cloned.asset_id))
            .filter(q_dsl::day.eq(&day_str))
            .filter(q_dsl::source.eq(&quote_cloned.data_source))
            .select(QuoteDB::as_select())
            .first::<QuoteDB>(&mut conn)
            .await
            .optional()
            .map_err(StoragePgError::from)?;

        let quote_id = if let Some(row) = existing {
            row.id.clone()
        } else {
            quote_cloned.id.clone()
        };

        let close_str = quote_cloned.close.to_string();
        let open_str = quote_cloned.open.to_string();
        let high_str = quote_cloned.high.to_string();
        let low_str = quote_cloned.low.to_string();
        let adjclose_str = quote_cloned.adjclose.to_string();
        let volume_str = quote_cloned.volume.to_string();

        // Clone strings for the ON CONFLICT SET clause (values() moves them)
        let open_str2 = open_str.clone();
        let high_str2 = high_str.clone();
        let low_str2 = low_str.clone();
        let adjclose_str2 = adjclose_str.clone();
        let volume_str2 = volume_str.clone();

        diesel::insert_into(q_dsl::quotes)
            .values((
                q_dsl::id.eq(&quote_id),
                q_dsl::asset_id.eq(&quote_cloned.asset_id),
                q_dsl::day.eq(&day_str),
                q_dsl::source.eq(&quote_cloned.data_source),
                q_dsl::open.eq(Some(open_str)),
                q_dsl::high.eq(Some(high_str)),
                q_dsl::low.eq(Some(low_str)),
                q_dsl::close.eq(&close_str),
                q_dsl::adjclose.eq(Some(adjclose_str)),
                q_dsl::volume.eq(Some(volume_str)),
                q_dsl::currency.eq(&quote_cloned.currency),
                q_dsl::notes.eq(&quote_cloned.notes),
                q_dsl::created_at.eq(now),
                q_dsl::timestamp.eq(quote_cloned.timestamp.naive_utc()),
            ))
            .on_conflict(q_dsl::id)
            .do_update()
            .set((
                q_dsl::close.eq(&close_str),
                q_dsl::open.eq(Some(open_str2)),
                q_dsl::high.eq(Some(high_str2)),
                q_dsl::low.eq(Some(low_str2)),
                q_dsl::adjclose.eq(Some(adjclose_str2)),
                q_dsl::volume.eq(Some(volume_str2)),
                q_dsl::source.eq(&quote_cloned.data_source),
            ))
            .execute(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        Ok(quote_cloned)
    }

    async fn delete_quote(&self, quote_id: &str) -> Result<()> {
        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;

        diesel::delete(q_dsl::quotes.filter(q_dsl::id.eq(quote_id)))
            .execute(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        Ok(())
    }

    async fn upsert_quotes(&self, input_quotes: &[Quote]) -> Result<usize> {
        if input_quotes.is_empty() {
            return Ok(0);
        }

        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;
        let now = chrono::Utc::now().naive_utc();

        // Check which days already have MANUAL quotes (they should not be overwritten)
        let asset_ids: Vec<&str> = input_quotes.iter().map(|q| q.asset_id.as_str()).collect();
        let days: Vec<String> = input_quotes.iter().map(|q| q.timestamp.format("%Y-%m-%d").to_string()).collect();

        let manual_days: std::collections::HashSet<(String, String)> = q_dsl::quotes
            .filter(q_dsl::source.eq("MANUAL"))
            .filter(q_dsl::asset_id.eq_any(&asset_ids))
            .filter(q_dsl::day.eq_any(&days))
            .select((q_dsl::asset_id, q_dsl::day))
            .load::<(String, String)>(&mut conn)
            .await
            .map_err(StoragePgError::from)?
            .into_iter()
            .collect();

        let mut total_upserted: usize = 0;

        for quote in input_quotes {
            let day_str = quote.timestamp.format("%Y-%m-%d").to_string();

            // Skip provider quotes for days that already have MANUAL override
            if quote.data_source != "MANUAL" && manual_days.contains(&(quote.asset_id.clone(), day_str.clone())) {
                continue;
            }

            let close_str = quote.close.to_string();
            let open_str = quote.open.to_string();
            let high_str = quote.high.to_string();
            let low_str = quote.low.to_string();
            let adjclose_str = quote.adjclose.to_string();
            let volume_str = quote.volume.to_string();

            // Clone for the ON CONFLICT SET clause (values() moves them)
            let open_str2 = open_str.clone();
            let high_str2 = high_str.clone();
            let low_str2 = low_str.clone();
            let adjclose_str2 = adjclose_str.clone();
            let volume_str2 = volume_str.clone();

            diesel::insert_into(q_dsl::quotes)
                .values((
                    q_dsl::id.eq(&quote.id),
                    q_dsl::asset_id.eq(&quote.asset_id),
                    q_dsl::day.eq(&day_str),
                    q_dsl::source.eq(&quote.data_source),
                    q_dsl::open.eq(Some(open_str)),
                    q_dsl::high.eq(Some(high_str)),
                    q_dsl::low.eq(Some(low_str)),
                    q_dsl::close.eq(&close_str),
                    q_dsl::adjclose.eq(Some(adjclose_str)),
                    q_dsl::volume.eq(Some(volume_str)),
                    q_dsl::currency.eq(&quote.currency),
                    q_dsl::notes.eq(&quote.notes),
                    q_dsl::created_at.eq(now),
                    q_dsl::timestamp.eq(quote.timestamp.naive_utc()),
                ))
                .on_conflict(q_dsl::id)
                .do_update()
                .set((
                    q_dsl::close.eq(&close_str),
                    q_dsl::open.eq(Some(open_str2)),
                    q_dsl::high.eq(Some(high_str2)),
                    q_dsl::low.eq(Some(low_str2)),
                    q_dsl::adjclose.eq(Some(adjclose_str2)),
                    q_dsl::volume.eq(Some(volume_str2)),
                    q_dsl::source.eq(&quote.data_source),
                ))
                .execute(&mut conn)
                .await
                .map_err(StoragePgError::from)?;

            total_upserted += 1;
        }

        Ok(total_upserted)
    }

    async fn delete_quotes_for_asset(&self, asset_id: &AssetId) -> Result<usize> {
        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;

        let count = diesel::delete(q_dsl::quotes.filter(q_dsl::asset_id.eq(asset_id.as_str())))
            .execute(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        Ok(count)
    }

    async fn delete_provider_quotes_for_asset(&self, asset_id: &AssetId) -> Result<usize> {
        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;

        let count = diesel::delete(
            q_dsl::quotes
                .filter(q_dsl::asset_id.eq(asset_id.as_str()))
                .filter(q_dsl::source.ne("MANUAL")),
        )
        .execute(&mut conn)
        .await
        .map_err(StoragePgError::from)?;

        Ok(count)
    }

    // =========================================================================
    // Single Asset Queries (Strong Types)
    // =========================================================================

    async fn latest(&self, asset_id: &AssetId, source: Option<&QuoteSource>) -> Result<Option<Quote>> {
        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;

        let mut query = q_dsl::quotes
            .filter(q_dsl::asset_id.eq(asset_id.as_str()))
            .order((
                q_dsl::day.desc(),
                diesel::dsl::sql::<diesel::sql_types::Integer>(SOURCE_PRIORITY_CASE).asc(),
            ))
            .into_boxed();

        if let Some(src) = source {
            query = query.filter(q_dsl::source.eq(src.to_storage_string()));
        }

        let result = query
            .first::<QuoteDB>(&mut conn)
            .await
            .optional()
            .map_err(StoragePgError::from)?;

        Ok(result.map(Quote::from))
    }

    async fn range(
        &self,
        asset_id: &AssetId,
        start: Day,
        end: Day,
        source: Option<&QuoteSource>,
    ) -> Result<Vec<Quote>> {
        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;

        let start_str = start.date().format("%Y-%m-%d").to_string();
        let end_str = end.date().format("%Y-%m-%d").to_string();

        let mut query = q_dsl::quotes
            .filter(q_dsl::asset_id.eq(asset_id.as_str()))
            .filter(q_dsl::day.ge(&start_str))
            .filter(q_dsl::day.le(&end_str))
            .order(q_dsl::day.asc())
            .into_boxed();

        if let Some(src) = source {
            query = query.filter(q_dsl::source.eq(src.to_storage_string()));
        }

        let results = query
            .load::<QuoteDB>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        Ok(results.into_iter().map(Quote::from).collect())
    }

    // =========================================================================
    // Batch Queries (Strong Types)
    // =========================================================================

    async fn latest_batch(
        &self,
        asset_ids: &[AssetId],
        source: Option<&QuoteSource>,
    ) -> Result<HashMap<AssetId, Quote>> {
        if asset_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;

        let ids: Vec<String> = asset_ids.iter().map(|id| id.as_str().to_string()).collect();

        // Use DISTINCT ON for efficient per-asset latest quote (PostgreSQL native)
        let results: Vec<QuoteDB> = if let Some(src) = source {
            let sql = format!(
                "SELECT DISTINCT ON (asset_id) \
                 id, asset_id, day, source, open, high, low, close, adjclose, volume, \
                 currency, notes, created_at, timestamp \
                 FROM quotes \
                 WHERE asset_id = ANY($1) AND source = $2 \
                 ORDER BY asset_id, day DESC, {SOURCE_PRIORITY_CASE} ASC"
            );
            diesel::sql_query(sql)
                .bind::<diesel::sql_types::Array<diesel::sql_types::Text>, Vec<String>>(ids)
                .bind::<diesel::sql_types::Text, String>(src.to_storage_string())
                .load::<QuoteDB>(&mut conn)
                .await
                .map_err(StoragePgError::from)?
        } else {
            let sql = format!(
                "SELECT DISTINCT ON (asset_id) \
                 id, asset_id, day, source, open, high, low, close, adjclose, volume, \
                 currency, notes, created_at, timestamp \
                 FROM quotes \
                 WHERE asset_id = ANY($1) \
                 ORDER BY asset_id, day DESC, {SOURCE_PRIORITY_CASE} ASC"
            );
            diesel::sql_query(sql)
                .bind::<diesel::sql_types::Array<diesel::sql_types::Text>, Vec<String>>(ids)
                .load::<QuoteDB>(&mut conn)
                .await
                .map_err(StoragePgError::from)?
        };

        let mut result_map = HashMap::new();
        for quote_db in results {
            let aid = AssetId::new(quote_db.asset_id.clone());
            result_map.insert(aid, Quote::from(quote_db));
        }

        Ok(result_map)
    }

    async fn latest_with_previous(
        &self,
        asset_ids: &[AssetId],
    ) -> Result<HashMap<AssetId, LatestQuotePair>> {
        if asset_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;

        let ids: Vec<String> = asset_ids.iter().map(|id| id.as_str().to_string()).collect();

        // Get top 2 quotes per asset using window function
        let sql = format!(
            "WITH RankedQuotes AS ( \
                SELECT \
                    id, asset_id, day, source, open, high, low, close, adjclose, volume, \
                    currency, notes, created_at, timestamp, \
                    ROW_NUMBER() OVER (PARTITION BY asset_id ORDER BY day DESC, {SOURCE_PRIORITY_CASE} ASC) as rn \
                FROM quotes WHERE asset_id = ANY($1) \
            ) \
            SELECT id, asset_id, day, source, open, high, low, close, adjclose, volume, \
                   currency, notes, created_at, timestamp \
            FROM RankedQuotes \
            WHERE rn <= 2 \
            ORDER BY asset_id, rn"
        );

        let ranked_quotes: Vec<QuoteDB> = diesel::sql_query(sql)
            .bind::<diesel::sql_types::Array<diesel::sql_types::Text>, Vec<String>>(ids)
            .load::<QuoteDB>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        let mut result_map: HashMap<AssetId, LatestQuotePair> = HashMap::new();
        let mut current_asset_quotes: Vec<Quote> = Vec::new();

        for quote_db in ranked_quotes {
            let quote = Quote::from(quote_db);

            if current_asset_quotes.is_empty()
                || quote.asset_id == current_asset_quotes[0].asset_id
            {
                current_asset_quotes.push(quote);
            } else {
                Self::flush_pair(&mut current_asset_quotes, &mut result_map);
                current_asset_quotes.push(quote);
            }
        }

        // Process final asset
        Self::flush_pair(&mut current_asset_quotes, &mut result_map);

        Ok(result_map)
    }

    async fn get_quote_bounds_for_assets(
        &self,
        asset_ids: &[String],
        source: &str,
    ) -> Result<HashMap<String, (NaiveDate, NaiveDate)>> {
        if asset_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;

        let sql = "SELECT asset_id, MIN(day) as min_day, MAX(day) as max_day \
                   FROM quotes \
                   WHERE asset_id = ANY($1) AND source = $2 \
                   GROUP BY asset_id";

        #[derive(QueryableByName, Debug)]
        struct QuoteBoundsRow {
            #[diesel(sql_type = diesel::sql_types::Text)]
            asset_id: String,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            min_day: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            max_day: Option<String>,
        }

        let rows: Vec<QuoteBoundsRow> = diesel::sql_query(sql)
            .bind::<diesel::sql_types::Array<diesel::sql_types::Text>, &[String]>(asset_ids)
            .bind::<diesel::sql_types::Text, &str>(source)
            .load::<QuoteBoundsRow>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        let mut result = HashMap::new();
        for row in rows {
            if let (Some(min_str), Some(max_str)) = (row.min_day, row.max_day) {
                if let (Ok(min_date), Ok(max_date)) = (
                    NaiveDate::parse_from_str(&min_str, "%Y-%m-%d"),
                    NaiveDate::parse_from_str(&max_str, "%Y-%m-%d"),
                ) {
                    result.insert(row.asset_id, (min_date, max_date));
                }
            }
        }

        Ok(result)
    }

    // =========================================================================
    // Legacy Methods (String-based)
    // =========================================================================

    async fn get_latest_quote(&self, symbol: &str) -> Result<Quote> {
        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;

        let query_result = q_dsl::quotes
            .filter(q_dsl::asset_id.eq(symbol))
            .order((
                q_dsl::day.desc(),
                diesel::dsl::sql::<diesel::sql_types::Integer>(SOURCE_PRIORITY_CASE).asc(),
            ))
            .first::<QuoteDB>(&mut conn)
            .await
            .optional()
            .map_err(StoragePgError::from)?;

        match query_result {
            Some(quote_db) => Ok(Quote::from(quote_db)),
            None => Err(whaleit_core::errors::Error::Database(
                DatabaseError::NotFound(format!("No quote found in database for symbol: {}", symbol)),
            )),
        }
    }

    async fn get_latest_quotes(&self, symbols: &[String]) -> Result<HashMap<String, Quote>> {
        if symbols.is_empty() {
            return Ok(HashMap::new());
        }

        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;

        let sql = format!(
            "SELECT DISTINCT ON (asset_id) \
             id, asset_id, day, source, open, high, low, close, adjclose, volume, \
             currency, notes, created_at, timestamp \
             FROM quotes \
             WHERE asset_id = ANY($1) \
             ORDER BY asset_id, day DESC, {SOURCE_PRIORITY_CASE} ASC"
        );

        let results: Vec<QuoteDB> = diesel::sql_query(sql)
            .bind::<diesel::sql_types::Array<diesel::sql_types::Text>, &[String]>(symbols)
            .load::<QuoteDB>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        let mut result = HashMap::new();
        for quote_db in results {
            result.insert(quote_db.asset_id.clone(), Quote::from(quote_db));
        }

        Ok(result)
    }

    async fn get_latest_quotes_pair(
        &self,
        symbols: &[String],
    ) -> Result<HashMap<String, LatestQuotePair>> {
        if symbols.is_empty() {
            return Ok(HashMap::new());
        }

        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;

        let sql = format!(
            "WITH RankedQuotes AS ( \
                SELECT \
                    id, asset_id, day, source, open, high, low, close, adjclose, volume, \
                    currency, notes, created_at, timestamp, \
                    ROW_NUMBER() OVER (PARTITION BY asset_id ORDER BY day DESC, {SOURCE_PRIORITY_CASE} ASC) as rn \
                FROM quotes WHERE asset_id = ANY($1) \
            ) \
            SELECT id, asset_id, day, source, open, high, low, close, adjclose, volume, \
                   currency, notes, created_at, timestamp \
            FROM RankedQuotes \
            WHERE rn <= 2 \
            ORDER BY asset_id, rn"
        );

        let ranked_quotes: Vec<QuoteDB> = diesel::sql_query(sql)
            .bind::<diesel::sql_types::Array<diesel::sql_types::Text>, &[String]>(symbols)
            .load::<QuoteDB>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        let mut result_map: HashMap<String, LatestQuotePair> = HashMap::new();
        let mut current_asset_quotes: Vec<Quote> = Vec::new();

        for quote_db in ranked_quotes {
            let quote = Quote::from(quote_db);

            if current_asset_quotes.is_empty()
                || quote.asset_id == current_asset_quotes[0].asset_id
            {
                current_asset_quotes.push(quote);
            } else {
                Self::flush_string_pair(&mut current_asset_quotes, &mut result_map);
                current_asset_quotes.push(quote);
            }
        }

        Self::flush_string_pair(&mut current_asset_quotes, &mut result_map);

        Ok(result_map)
    }

    async fn get_historical_quotes(&self, symbol: &str) -> Result<Vec<Quote>> {
        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;

        let results = q_dsl::quotes
            .filter(q_dsl::asset_id.eq(symbol))
            .order(q_dsl::day.desc())
            .load::<QuoteDB>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        Ok(results.into_iter().map(Quote::from).collect())
    }

    async fn get_all_historical_quotes(&self) -> Result<Vec<Quote>> {
        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;

        let results = q_dsl::quotes
            .order(q_dsl::day.desc())
            .load::<QuoteDB>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        Ok(results.into_iter().map(Quote::from).collect())
    }

    async fn get_quotes_in_range(
        &self,
        symbol: &str,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<Quote>> {
        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;

        let start_str = start.format("%Y-%m-%d").to_string();
        let end_str = end.format("%Y-%m-%d").to_string();

        let results = q_dsl::quotes
            .filter(q_dsl::asset_id.eq(symbol))
            .filter(q_dsl::day.ge(&start_str))
            .filter(q_dsl::day.le(&end_str))
            .order(q_dsl::day.asc())
            .load::<QuoteDB>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        Ok(results.into_iter().map(Quote::from).collect())
    }

    async fn find_duplicate_quotes(&self, symbol: &str, date: NaiveDate) -> Result<Vec<Quote>> {
        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;

        let date_str = date.format("%Y-%m-%d").to_string();

        let results = q_dsl::quotes
            .filter(q_dsl::asset_id.eq(symbol))
            .filter(q_dsl::day.eq(&date_str))
            .load::<QuoteDB>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        Ok(results.into_iter().map(Quote::from).collect())
    }
}

// =============================================================================
// ProviderSettingsStore Implementation
// =============================================================================

#[async_trait]
impl ProviderSettingsStore for PgMarketDataRepository {
    async fn get_all_providers(&self) -> Result<Vec<MarketDataProviderSetting>> {
        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;

        let db_results = mdp_dsl::market_data_providers
            .order(mdp_dsl::priority.desc())
            .select(MarketDataProviderSettingDB::as_select())
            .load::<MarketDataProviderSettingDB>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        Ok(db_results.into_iter().map(MarketDataProviderSetting::from).collect())
    }

    async fn get_provider(&self, id: &str) -> Result<MarketDataProviderSetting> {
        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;

        let db_result = mdp_dsl::market_data_providers
            .find(id)
            .select(MarketDataProviderSettingDB::as_select())
            .first::<MarketDataProviderSettingDB>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        Ok(MarketDataProviderSetting::from(db_result))
    }

    async fn update_provider(
        &self,
        id: &str,
        changes: UpdateMarketDataProviderSetting,
    ) -> Result<MarketDataProviderSetting> {
        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;

        let changes_db = UpdateMarketDataProviderSettingDB {
            priority: changes.priority,
            enabled: changes.enabled,
            config: None,
        };

        // Use AsChangeset derive — None fields won't be included in UPDATE SET
        diesel::update(mdp_dsl::market_data_providers.find(id))
            .set(&changes_db)
            .execute(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        let db_result = mdp_dsl::market_data_providers
            .find(id)
            .select(MarketDataProviderSettingDB::as_select())
            .first::<MarketDataProviderSettingDB>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        Ok(MarketDataProviderSetting::from(db_result))
    }
}

// =============================================================================
// Helper methods
// =============================================================================

impl PgMarketDataRepository {
    fn flush_pair(
        current: &mut Vec<Quote>,
        result_map: &mut HashMap<AssetId, LatestQuotePair>,
    ) {
        if current.is_empty() {
            return;
        }
        let latest_quote = current.remove(0);
        let previous_quote = if !current.is_empty() {
            Some(current.remove(0))
        } else {
            None
        };
        result_map.insert(
            AssetId::new(latest_quote.asset_id.clone()),
            LatestQuotePair {
                latest: latest_quote,
                previous: previous_quote,
            },
        );
        current.clear();
    }

    fn flush_string_pair(
        current: &mut Vec<Quote>,
        result_map: &mut HashMap<String, LatestQuotePair>,
    ) {
        if current.is_empty() {
            return;
        }
        let latest_quote = current.remove(0);
        let previous_quote = if !current.is_empty() {
            Some(current.remove(0))
        } else {
            None
        };
        result_map.insert(
            latest_quote.asset_id.clone(),
            LatestQuotePair {
                latest: latest_quote,
                previous: previous_quote,
            },
        );
        current.clear();
    }
}
