//! PostgreSQL FX repository implementation.

use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use std::sync::Arc;

use super::model::{NewQuoteDB, QuoteDB};
use crate::assets::{AssetDB, InsertableAssetDB};
use crate::db::PgPool;
use crate::errors::StoragePgError;
use crate::schema::assets;
use crate::schema::quotes::dsl as q_dsl;
use whaleit_core::errors::{DatabaseError, Result};
use whaleit_core::fx::{ExchangeRate, FxRepositoryTrait};
use whaleit_core::quotes::Quote;
use chrono::{NaiveDateTime, TimeZone, Utc};
use rust_decimal::Decimal;
use std::str::FromStr;

pub struct PgFxRepository {
    pool: Arc<PgPool>,
}

impl PgFxRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

/// Helper to build an ExchangeRate from a joined (QuoteDB, AssetDB) row.
fn build_exchange_rate(quote_db: &QuoteDB, asset_db: &AssetDB) -> ExchangeRate {
    let timestamp = Utc.from_utc_datetime(&quote_db.timestamp);
    let rate = Decimal::from_str(&quote_db.close).unwrap_or(Decimal::ZERO);
    let from_currency = asset_db.instrument_symbol.clone().unwrap_or_default();

    ExchangeRate {
        id: asset_db.id.clone(),
        from_currency,
        to_currency: asset_db.quote_ccy.clone(),
        rate,
        source: quote_db.source.clone(),
        timestamp,
    }
}

#[async_trait]
impl FxRepositoryTrait for PgFxRepository {
    async fn get_latest_exchange_rates(&self) -> Result<Vec<ExchangeRate>> {
        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;

        // Get all FX assets
        let forex_assets: Vec<AssetDB> = assets::table
            .filter(assets::kind.eq("FX"))
            .order(assets::display_code.asc())
            .select(AssetDB::as_select())
            .load(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        if forex_assets.is_empty() {
            return Ok(vec![]);
        }

        let asset_ids: Vec<String> = forex_assets.iter().map(|a| a.id.clone()).collect();

        // Get latest quote per FX asset using PostgreSQL DISTINCT ON
        let latest_quotes: Vec<QuoteDB> = diesel::sql_query(
            "SELECT DISTINCT ON (asset_id) \
             id, asset_id, day, source, open, high, low, close, adjclose, volume, \
             currency, notes, created_at, timestamp \
             FROM quotes \
             WHERE asset_id = ANY($1) \
             ORDER BY asset_id, day DESC, CASE source WHEN 'MANUAL' THEN 1 WHEN 'BROKER' THEN 2 ELSE 3 END ASC"
        )
        .bind::<diesel::sql_types::Array<diesel::sql_types::Text>, Vec<String>>(asset_ids)
        .load::<QuoteDB>(&mut conn)
        .await
        .map_err(StoragePgError::from)?;

        let quote_by_asset_id: std::collections::HashMap<String, QuoteDB> = latest_quotes
            .into_iter()
            .map(|q| (q.asset_id.clone(), q))
            .collect();

        let mut rates = Vec::with_capacity(forex_assets.len());
        for asset in &forex_assets {
            if let Some(quote_db) = quote_by_asset_id.get(&asset.id) {
                rates.push(build_exchange_rate(quote_db, asset));
            } else {
                // No quote found — still include asset with zero rate
                let ts = Utc.from_utc_datetime(&asset.updated_at);
                let source = asset
                    .provider_config
                    .as_ref()
                    .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
                    .and_then(|v| v.get("preferred_provider")?.as_str().map(String::from))
                    .unwrap_or_else(|| "MANUAL".to_string());

                rates.push(ExchangeRate {
                    id: asset.id.clone(),
                    from_currency: asset.instrument_symbol.clone().unwrap_or_default(),
                    to_currency: asset.quote_ccy.clone(),
                    rate: Decimal::ZERO,
                    source,
                    timestamp: ts,
                });
            }
        }

        Ok(rates)
    }

    async fn get_historical_exchange_rates(&self) -> Result<Vec<ExchangeRate>> {
        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;

        let results: Vec<(QuoteDB, AssetDB)> = q_dsl::quotes
            .inner_join(assets::table.on(q_dsl::asset_id.eq(assets::id)))
            .filter(assets::kind.eq("FX"))
            .select((QuoteDB::as_select(), AssetDB::as_select()))
            .order((q_dsl::asset_id.asc(), q_dsl::timestamp.asc()))
            .load::<(QuoteDB, AssetDB)>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        Ok(results
            .into_iter()
            .map(|(q, a)| build_exchange_rate(&q, &a))
            .collect())
    }

    async fn get_latest_exchange_rate(&self, from: &str, _to: &str) -> Result<Option<ExchangeRate>> {
        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;

        let expected_key = format!("FX:{}/{}", from, _to);

        let result: Option<(QuoteDB, AssetDB)> = q_dsl::quotes
            .inner_join(assets::table.on(q_dsl::asset_id.eq(assets::id)))
            .filter(assets::instrument_key.eq(&expected_key))
            .order_by(q_dsl::timestamp.desc())
            .select((QuoteDB::as_select(), AssetDB::as_select()))
            .first::<(QuoteDB, AssetDB)>(&mut conn)
            .await
            .optional()
            .map_err(StoragePgError::from)?;

        Ok(result.map(|(q, a)| build_exchange_rate(&q, &a)))
    }

    async fn get_latest_exchange_rate_by_symbol(&self, symbol: &str) -> Result<Option<ExchangeRate>> {
        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;

        let result: Option<(QuoteDB, AssetDB)> = q_dsl::quotes
            .inner_join(assets::table.on(q_dsl::asset_id.eq(assets::id)))
            .filter(
                assets::instrument_key
                    .eq(symbol)
                    .or(q_dsl::asset_id.eq(symbol)),
            )
            .order_by(q_dsl::timestamp.desc())
            .select((QuoteDB::as_select(), AssetDB::as_select()))
            .first::<(QuoteDB, AssetDB)>(&mut conn)
            .await
            .optional()
            .map_err(StoragePgError::from)?;

        Ok(result.map(|(q, a)| build_exchange_rate(&q, &a)))
    }

    async fn get_historical_quotes(
        &self,
        symbol: &str,
        start_date: NaiveDateTime,
        end_date: NaiveDateTime,
    ) -> Result<Vec<Quote>> {
        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;

        // symbol is an instrument_key (e.g., "FX:EUR/USD") or asset_id
        let asset_ids: Vec<String> = assets::table
            .filter(
                assets::instrument_key
                    .eq(symbol)
                    .or(assets::id.eq(symbol)),
            )
            .select(assets::id)
            .load::<String>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        if asset_ids.is_empty() {
            return Ok(vec![]);
        }

        let results: Vec<QuoteDB> = q_dsl::quotes
            .filter(q_dsl::asset_id.eq_any(&asset_ids))
            .filter(q_dsl::timestamp.ge(start_date))
            .filter(q_dsl::timestamp.le(end_date))
            .order(q_dsl::timestamp.asc())
            .load::<QuoteDB>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        Ok(results.into_iter().map(Quote::from).collect())
    }

    async fn add_quote(
        &self,
        symbol: String,
        date_str: String,
        rate: Decimal,
        source_str: String,
    ) -> Result<Quote> {
        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;

        // Look up the asset to get currency
        let asset: AssetDB = assets::table
            .filter(assets::id.eq(&symbol))
            .select(AssetDB::as_select())
            .first::<AssetDB>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        let currency = asset.instrument_symbol.clone().unwrap_or_default();
        let rate_str = rate.to_string();
        let quote_id = format!("{}_{}_{}", symbol, date_str, source_str);

        let naive_date = chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").map_err(|e| {
            whaleit_core::errors::Error::Unexpected(format!("Invalid date format: {}", e))
        })?;
        let naive_datetime = naive_date
            .and_hms_opt(16, 0, 0)
            .ok_or_else(|| {
                whaleit_core::errors::Error::Unexpected(format!(
                    "Failed to create NaiveDateTime for {}",
                    date_str
                ))
            })?;
        let timestamp_utc = Utc.from_utc_datetime(&naive_datetime);
        let now = Utc::now();

        let new_quote = NewQuoteDB {
            id: quote_id.clone(),
            asset_id: symbol.clone(),
            day: date_str.clone(),
            source: source_str.clone(),
            open: Some(rate_str.clone()),
            high: Some(rate_str.clone()),
            low: Some(rate_str.clone()),
            close: rate_str.clone(),
            adjclose: Some(rate_str.clone()),
            volume: None,
            currency,
            notes: None,
            created_at: now.naive_utc(),
            timestamp: timestamp_utc.naive_utc(),
        };

        // UPSERT using ON CONFLICT
        diesel::insert_into(q_dsl::quotes)
            .values(&new_quote)
            .on_conflict(q_dsl::id)
            .do_update()
            .set((
                q_dsl::close.eq(rate_str.clone()),
                q_dsl::source.eq(source_str.clone()),
            ))
            .execute(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        // Read back
        let inserted: QuoteDB = q_dsl::quotes
            .filter(q_dsl::asset_id.eq(&symbol))
            .filter(q_dsl::day.eq(&date_str))
            .filter(q_dsl::source.eq(&source_str))
            .select(QuoteDB::as_select())
            .first::<QuoteDB>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        Ok(Quote::from(inserted))
    }

    async fn save_exchange_rate(&self, rate: ExchangeRate) -> Result<ExchangeRate> {
        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;

        let quote = rate.to_quote();
        let new_quote = NewQuoteDB::from(&quote);

        diesel::insert_into(q_dsl::quotes)
            .values(&new_quote)
            .on_conflict(q_dsl::id)
            .do_update()
            .set((
                q_dsl::open.eq(new_quote.open.clone()),
                q_dsl::high.eq(new_quote.high.clone()),
                q_dsl::low.eq(new_quote.low.clone()),
                q_dsl::close.eq(new_quote.close.clone()),
                q_dsl::adjclose.eq(new_quote.adjclose.clone()),
                q_dsl::volume.eq(new_quote.volume.clone()),
                q_dsl::source.eq(new_quote.source.clone()),
            ))
            .execute(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        Ok(rate)
    }

    async fn update_exchange_rate(&self, rate: &ExchangeRate) -> Result<ExchangeRate> {
        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;

        let quote = rate.to_quote();
        let day_str = quote.timestamp.format("%Y-%m-%d").to_string();
        let close_str = quote.close.to_string();
        let source_str = quote.data_source.clone();

        let updated = diesel::update(q_dsl::quotes)
            .filter(q_dsl::asset_id.eq(&quote.asset_id))
            .filter(q_dsl::day.eq(&day_str))
            .set((
                q_dsl::close.eq(close_str),
                q_dsl::source.eq(source_str),
            ))
            .execute(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        if updated == 0 {
            return Err(whaleit_core::errors::Error::Database(DatabaseError::NotFound(
                format!("Exchange rate quote not found for asset {}", quote.asset_id),
            )));
        }

        Ok(rate.clone())
    }

    async fn delete_exchange_rate(&self, rate_id: &str) -> Result<()> {
        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;

        // Delete quotes for this asset
        diesel::delete(q_dsl::quotes.filter(q_dsl::asset_id.eq(rate_id)))
            .execute(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        // Delete the asset itself
        diesel::delete(assets::table.filter(assets::id.eq(rate_id)))
            .execute(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        Ok(())
    }

    async fn create_fx_asset(
        &self,
        from_currency: &str,
        to_currency: &str,
        _source: &str,
    ) -> Result<String> {
        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;

        let expected_key = format!("FX:{}/{}", from_currency, to_currency);

        // Check if already exists
        let existing: Option<AssetDB> = assets::table
            .filter(assets::instrument_key.eq(&expected_key))
            .select(AssetDB::as_select())
            .first::<AssetDB>(&mut conn)
            .await
            .optional()
            .map_err(StoragePgError::from)?;

        if let Some(asset) = existing {
            return Ok(asset.id);
        }

        // Create new FX asset
        let now = Utc::now().naive_utc();
        let new_id = uuid::Uuid::now_v7().to_string();
        let display_code = format!("{}/{}", from_currency, to_currency);
        let instrument_symbol = Some(from_currency.to_string());

        let new_asset = InsertableAssetDB {
            id: Some(new_id.clone()),
            kind: "FX".to_string(),
            name: Some(display_code.clone()),
            display_code: Some(display_code),
            notes: None,
            metadata: None,
            is_active: true,
            quote_mode: "MARKET".to_string(),
            quote_ccy: to_currency.to_string(),
            instrument_type: Some("CURRENCY".to_string()),
            instrument_symbol,
            instrument_exchange_mic: None,
            provider_config: None,
            created_at: now,
            updated_at: now,
        };

        diesel::insert_into(assets::table)
            .values(&new_asset)
            .execute(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        Ok(new_id)
    }
}
