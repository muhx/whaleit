//! PostgreSQL market data repository (QuoteStore + ProviderSettingsStore).

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

use crate::db::PgPool;
use whaleit_core::errors::Result;
use whaleit_core::quotes::{
    LatestQuotePair, MarketDataProviderSetting, ProviderSettingsStore, Quote, QuoteStore,
    UpdateMarketDataProviderSetting,
};
use whaleit_core::quotes::types::{AssetId, Day, QuoteSource};
use chrono::NaiveDate;

pub struct PgMarketDataRepository {
    #[allow(dead_code)]
    pool: Arc<PgPool>,
}

impl PgMarketDataRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl QuoteStore for PgMarketDataRepository {
    async fn save_quote(&self, _quote: &Quote) -> Result<Quote> {
        Err(whaleit_core::errors::Error::Unexpected("not yet implemented".to_string()))
    }
    async fn delete_quote(&self, _quote_id: &str) -> Result<()> { Ok(()) }
    async fn upsert_quotes(&self, _quotes: &[Quote]) -> Result<usize> { Ok(0) }
    async fn delete_quotes_for_asset(&self, _asset_id: &AssetId) -> Result<usize> { Ok(0) }
    async fn delete_provider_quotes_for_asset(&self, _asset_id: &AssetId) -> Result<usize> { Ok(0) }
    async fn latest(&self, _asset_id: &AssetId, _source: Option<&QuoteSource>) -> Result<Option<Quote>> { Ok(None) }
    async fn range(&self, _asset_id: &AssetId, _start: Day, _end: Day, _source: Option<&QuoteSource>) -> Result<Vec<Quote>> { Ok(vec![]) }
    async fn latest_batch(&self, _asset_ids: &[AssetId], _source: Option<&QuoteSource>) -> Result<HashMap<AssetId, Quote>> { Ok(HashMap::new()) }
    async fn latest_with_previous(&self, _asset_ids: &[AssetId]) -> Result<HashMap<AssetId, LatestQuotePair>> { Ok(HashMap::new()) }
    async fn get_quote_bounds_for_assets(&self, _asset_ids: &[String], _source: &str) -> Result<HashMap<String, (NaiveDate, NaiveDate)>> { Ok(HashMap::new()) }
    async fn get_latest_quote(&self, _symbol: &str) -> Result<Quote> {
        Err(whaleit_core::errors::Error::Unexpected("not yet implemented".to_string()))
    }
    async fn get_latest_quotes(&self, _symbols: &[String]) -> Result<HashMap<String, Quote>> { Ok(HashMap::new()) }
    async fn get_latest_quotes_pair(&self, _symbols: &[String]) -> Result<HashMap<String, LatestQuotePair>> { Ok(HashMap::new()) }
    async fn get_historical_quotes(&self, _symbol: &str) -> Result<Vec<Quote>> { Ok(vec![]) }
    async fn get_all_historical_quotes(&self) -> Result<Vec<Quote>> { Ok(vec![]) }
    async fn get_quotes_in_range(&self, _symbol: &str, _start: NaiveDate, _end: NaiveDate) -> Result<Vec<Quote>> { Ok(vec![]) }
    async fn find_duplicate_quotes(&self, _symbol: &str, _date: NaiveDate) -> Result<Vec<Quote>> { Ok(vec![]) }
}

#[async_trait]
impl ProviderSettingsStore for PgMarketDataRepository {
    async fn get_all_providers(&self) -> Result<Vec<MarketDataProviderSetting>> { Ok(vec![]) }
    async fn get_provider(&self, _id: &str) -> Result<MarketDataProviderSetting> {
        Err(whaleit_core::errors::Error::Database(whaleit_core::errors::DatabaseError::NotFound("Provider not found".to_string())))
    }
    async fn update_provider(&self, _id: &str, _changes: UpdateMarketDataProviderSetting) -> Result<MarketDataProviderSetting> {
        Err(whaleit_core::errors::Error::Unexpected("not yet implemented".to_string()))
    }
}
