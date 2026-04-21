//! PostgreSQL snapshot repository implementation.

use async_trait::async_trait;
use chrono::NaiveDate;
use std::collections::HashMap;
use std::sync::Arc;

use crate::db::PgPool;
use whaleit_core::errors::Result;
use whaleit_core::portfolio::snapshot::{AccountStateSnapshot, SnapshotRepositoryTrait};

pub struct PgSnapshotRepository {
    #[allow(dead_code)]
    pool: Arc<PgPool>,
}

impl PgSnapshotRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SnapshotRepositoryTrait for PgSnapshotRepository {
    async fn save_snapshots(&self, _snapshots: &[AccountStateSnapshot]) -> Result<()> { Ok(()) }
    async fn get_snapshots_by_account(&self, _account_id: &str, _start_date: Option<NaiveDate>, _end_date: Option<NaiveDate>) -> Result<Vec<AccountStateSnapshot>> { Ok(vec![]) }
    async fn get_latest_snapshot_before_date(&self, _account_id: &str, _date: NaiveDate) -> Result<Option<AccountStateSnapshot>> { Ok(None) }
    async fn get_latest_snapshots_before_date(&self, _account_ids: &[String], _date: NaiveDate) -> Result<HashMap<String, AccountStateSnapshot>> { Ok(HashMap::new()) }
    async fn get_all_latest_snapshots(&self, _account_ids: &[String]) -> Result<HashMap<String, AccountStateSnapshot>> { Ok(HashMap::new()) }
    async fn delete_snapshots_by_account_ids(&self, _account_ids: &[String]) -> Result<usize> { Ok(0) }
    async fn delete_snapshots_for_account_and_dates(&self, _account_id: &str, _dates_to_delete: &[NaiveDate]) -> Result<()> { Ok(()) }
    async fn delete_snapshots_for_account_in_range(&self, _account_id: &str, _start_date: NaiveDate, _end_date: NaiveDate) -> Result<()> { Ok(()) }
    async fn overwrite_snapshots_for_account_in_range(&self, _account_id: &str, _start_date: NaiveDate, _end_date: NaiveDate, _snapshots_to_save: &[AccountStateSnapshot]) -> Result<()> { Ok(()) }
    async fn overwrite_multiple_account_snapshot_ranges(&self, _new_snapshots: &[AccountStateSnapshot]) -> Result<()> { Ok(()) }
    async fn get_total_portfolio_snapshots(&self, _start_date: Option<NaiveDate>, _end_date: Option<NaiveDate>) -> Result<Vec<AccountStateSnapshot>> { Ok(vec![]) }
    async fn get_all_non_archived_account_snapshots(&self, _start_date: Option<NaiveDate>, _end_date: Option<NaiveDate>) -> Result<Vec<AccountStateSnapshot>> { Ok(vec![]) }
    async fn get_earliest_snapshot_date(&self, _account_id: &str) -> Result<Option<NaiveDate>> { Ok(None) }
    async fn overwrite_all_snapshots_for_account(&self, _account_id: &str, _snapshots_to_save: &[AccountStateSnapshot]) -> Result<()> { Ok(()) }
    async fn update_snapshots_source(&self, _account_id: &str, _new_source: &str) -> Result<usize> { Ok(0) }
    async fn save_or_update_snapshot(&self, _snapshot: &AccountStateSnapshot) -> Result<()> { Ok(()) }
    async fn get_non_calculated_snapshot_count(&self, _account_id: &str) -> Result<usize> { Ok(0) }
    async fn get_earliest_non_calculated_snapshot(&self, _account_id: &str) -> Result<Option<AccountStateSnapshot>> { Ok(None) }
}
