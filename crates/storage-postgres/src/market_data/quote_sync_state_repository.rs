//! PostgreSQL quote sync state repository.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

use crate::db::PgPool;
use whaleit_core::errors::Result;
use whaleit_core::quotes::sync_state::{
    ProviderSyncStats, QuoteSyncState, SyncStateStore,
};
use chrono::NaiveDate;

pub struct PgQuoteSyncStateRepository {
    #[allow(dead_code)]
    pool: Arc<PgPool>,
}

impl PgQuoteSyncStateRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SyncStateStore for PgQuoteSyncStateRepository {
    async fn get_provider_sync_stats(&self) -> Result<Vec<ProviderSyncStats>> { Ok(vec![]) }
    async fn get_all(&self) -> Result<Vec<QuoteSyncState>> { Ok(vec![]) }
    async fn get_by_asset_id(&self, _asset_id: &str) -> Result<Option<QuoteSyncState>> { Ok(None) }
    async fn get_by_asset_ids(&self, _asset_ids: &[String]) -> Result<HashMap<String, QuoteSyncState>> { Ok(HashMap::new()) }
    async fn get_active_assets(&self) -> Result<Vec<QuoteSyncState>> { Ok(vec![]) }
    async fn get_assets_needing_sync(&self, _grace_period_days: i64) -> Result<Vec<QuoteSyncState>> { Ok(vec![]) }
    async fn upsert(&self, _state: &QuoteSyncState) -> Result<QuoteSyncState> {
        Err(whaleit_core::errors::Error::Unexpected("not yet implemented".to_string()))
    }
    async fn upsert_batch(&self, _states: &[QuoteSyncState]) -> Result<usize> { Ok(0) }
    async fn update_after_sync(&self, _asset_id: &str) -> Result<()> { Ok(()) }
    async fn update_after_failure(&self, _asset_id: &str, _error: &str) -> Result<()> { Ok(()) }
    async fn mark_inactive(&self, _asset_id: &str, _closed_date: NaiveDate) -> Result<()> { Ok(()) }
    async fn mark_active(&self, _asset_id: &str) -> Result<()> { Ok(()) }
    async fn delete(&self, _asset_id: &str) -> Result<()> { Ok(()) }
    async fn delete_all(&self) -> Result<usize> { Ok(0) }
    async fn mark_profile_enriched(&self, _asset_id: &str) -> Result<()> { Ok(()) }
    async fn get_assets_needing_profile_enrichment(&self) -> Result<Vec<QuoteSyncState>> { Ok(vec![]) }
    async fn get_with_errors(&self) -> Result<Vec<QuoteSyncState>> { Ok(vec![]) }
}
