//! PostgreSQL valuation repository implementation.

use async_trait::async_trait;
use chrono::NaiveDate;
use std::sync::Arc;

use crate::db::PgPool;
use whaleit_core::errors::Result;
use whaleit_core::portfolio::valuation::{DailyAccountValuation, NegativeBalanceInfo, ValuationRepositoryTrait};

pub struct PgValuationRepository {
    #[allow(dead_code)]
    pool: Arc<PgPool>,
}

impl PgValuationRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ValuationRepositoryTrait for PgValuationRepository {
    async fn save_valuations(&self, _valuation_records: &[DailyAccountValuation]) -> Result<()> { Ok(()) }
    async fn get_historical_valuations(&self, _account_id: &str, _start_date: Option<NaiveDate>, _end_date: Option<NaiveDate>) -> Result<Vec<DailyAccountValuation>> { Ok(vec![]) }
    async fn load_latest_valuation_date(&self, _account_id: &str) -> Result<Option<NaiveDate>> { Ok(None) }
    async fn delete_valuations_for_account(&self, _account_id: &str, _since_date: Option<NaiveDate>) -> Result<()> { Ok(()) }
    async fn get_latest_valuations(&self, _account_ids: &[String]) -> Result<Vec<DailyAccountValuation>> { Ok(vec![]) }
    async fn get_valuations_on_date(&self, _account_ids: &[String], _date: NaiveDate) -> Result<Vec<DailyAccountValuation>> { Ok(vec![]) }
    async fn get_accounts_with_negative_balance(&self, _account_ids: &[String]) -> Result<Vec<NegativeBalanceInfo>> { Ok(vec![]) }
}
