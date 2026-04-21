//! PostgreSQL FX repository implementation.

use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use std::sync::Arc;

use crate::db::PgPool;
use crate::errors::StoragePgError;
use crate::schema::quotes;
use whaleit_core::errors::Result;
use whaleit_core::fx::{ExchangeRate, FxRepositoryTrait, NewExchangeRate};
use whaleit_core::quotes::Quote;
use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;

pub struct PgFxRepository {
    pool: Arc<PgPool>,
}

impl PgFxRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl FxRepositoryTrait for PgFxRepository {
    async fn get_latest_exchange_rates(&self) -> Result<Vec<ExchangeRate>> {
        Ok(vec![])
    }

    async fn get_historical_exchange_rates(&self) -> Result<Vec<ExchangeRate>> {
        Ok(vec![])
    }

    async fn get_latest_exchange_rate(&self, _from: &str, _to: &str) -> Result<Option<ExchangeRate>> {
        Ok(None)
    }

    async fn get_latest_exchange_rate_by_symbol(&self, _symbol: &str) -> Result<Option<ExchangeRate>> {
        Ok(None)
    }

    async fn get_historical_quotes(
        &self,
        _symbol: &str,
        _start_date: NaiveDateTime,
        _end_date: NaiveDateTime,
    ) -> Result<Vec<Quote>> {
        Ok(vec![])
    }

    async fn add_quote(
        &self,
        _symbol: String,
        _date: String,
        _rate: Decimal,
        _source: String,
    ) -> Result<Quote> {
        Err(whaleit_core::errors::Error::Unexpected("not yet implemented".to_string()))
    }

    async fn save_exchange_rate(&self, _rate: ExchangeRate) -> Result<ExchangeRate> {
        Err(whaleit_core::errors::Error::Unexpected("not yet implemented".to_string()))
    }

    async fn update_exchange_rate(&self, _rate: &ExchangeRate) -> Result<ExchangeRate> {
        Err(whaleit_core::errors::Error::Unexpected("not yet implemented".to_string()))
    }

    async fn delete_exchange_rate(&self, _rate_id: &str) -> Result<()> {
        Ok(())
    }

    async fn create_fx_asset(
        &self,
        _from_currency: &str,
        _to_currency: &str,
        _source: &str,
    ) -> Result<String> {
        Err(whaleit_core::errors::Error::Unexpected("not yet implemented".to_string()))
    }
}
