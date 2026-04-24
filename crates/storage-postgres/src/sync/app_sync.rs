//! PostgreSQL app sync repository (stub implementation for compilation).
//!
//! NOTE: This provides correct method signatures and return types so the server
//! compiles with `--features postgres`. Full sync functionality uses stub bodies
//! that return errors for unsupported operations.

use std::sync::Arc;

use whaleit_core::errors::{DatabaseError, Error, Result};
use whaleit_core::sync::{
    SyncEngineStatus, SyncEntity, SyncEntityMetadata, SyncOperation, SyncOutboxEvent,
};

use crate::db::PgPool;

/// Row count result for sync table summary queries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncTableRowCount {
    pub table: String,
    pub rows: i64,
}

/// Summary of local sync data across all tables.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncLocalDataSummary {
    pub total_rows: i64,
    pub non_empty_tables: Vec<SyncTableRowCount>,
}

pub struct PgAppSyncRepository {
    #[allow(dead_code)]
    pool: Arc<PgPool>,
}

impl PgAppSyncRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    // -----------------------------------------------------------------------
    // Simple sync queries (return default/stub values)
    // -----------------------------------------------------------------------

    pub fn get_engine_status(&self) -> Result<SyncEngineStatus> {
        Ok(SyncEngineStatus {
            cursor: self.get_cursor()?,
            last_push_at: None,
            last_pull_at: None,
            last_error: None,
            consecutive_failures: 0,
            next_retry_at: None,
            last_cycle_status: None,
            last_cycle_duration_ms: None,
        })
    }

    pub fn needs_bootstrap(&self, _device_id: &str) -> Result<bool> {
        Ok(true)
    }

    pub fn get_cursor(&self) -> Result<i64> {
        Ok(0)
    }

    pub fn get_local_sync_data_summary(&self) -> Result<SyncLocalDataSummary> {
        Ok(SyncLocalDataSummary {
            total_rows: 0,
            non_empty_tables: Vec::new(),
        })
    }

    pub fn get_min_snapshot_created_at(&self, _device_id: &str) -> Result<Option<String>> {
        Ok(None)
    }

    pub fn list_pending_outbox(&self, _limit: i64) -> Result<Vec<SyncOutboxEvent>> {
        Ok(Vec::new())
    }

    pub fn get_entity_metadata(
        &self,
        _entity: SyncEntity,
        _entity_id: &str,
    ) -> Result<Option<SyncEntityMetadata>> {
        Ok(None)
    }

    pub fn has_pending_outbox(&self) -> Result<bool> {
        Ok(false)
    }

    // -----------------------------------------------------------------------
    // Async write methods (stubs that succeed silently)
    // -----------------------------------------------------------------------

    pub async fn set_cursor(&self, _cursor: i64) -> Result<()> {
        Ok(())
    }

    pub async fn upsert_device_config(
        &self,
        _device_id: String,
        _key_version: Option<i32>,
        _trust_state: String,
    ) -> Result<()> {
        Ok(())
    }

    pub async fn reset_local_sync_session(&self) -> Result<()> {
        Ok(())
    }

    pub async fn reset_and_mark_bootstrap_complete(
        &self,
        _device_id: String,
        _key_version: Option<i32>,
    ) -> Result<()> {
        Ok(())
    }

    pub async fn set_min_snapshot_created_at(
        &self,
        _device_id: String,
        _value: String,
    ) -> Result<()> {
        Ok(())
    }

    pub async fn clear_min_snapshot_created_at(&self, _device_id: String) -> Result<()> {
        Ok(())
    }

    pub async fn clear_all_min_snapshot_created_at(&self) -> Result<()> {
        Ok(())
    }

    pub async fn mark_outbox_sent(&self, _event_ids: Vec<String>) -> Result<()> {
        Ok(())
    }

    pub async fn schedule_outbox_retry(
        &self,
        _event_ids: Vec<String>,
        _backoff_seconds: i64,
        _last_error: Option<String>,
        _last_error_code: Option<String>,
    ) -> Result<()> {
        Ok(())
    }

    pub async fn mark_outbox_dead(
        &self,
        _event_ids: Vec<String>,
        _error_message: Option<String>,
        _error_code: Option<String>,
    ) -> Result<()> {
        Ok(())
    }

    pub async fn mark_push_completed(&self) -> Result<()> {
        Ok(())
    }

    pub async fn upsert_entity_metadata(&self, _metadata: SyncEntityMetadata) -> Result<()> {
        Ok(())
    }

    pub async fn acquire_cycle_lock(&self) -> Result<i64> {
        Ok(1)
    }

    pub async fn verify_cycle_lock(&self, _lock_version: i64) -> Result<bool> {
        Ok(true)
    }

    pub async fn apply_remote_event_lww(
        &self,
        _entity: SyncEntity,
        _entity_id: String,
        _op: SyncOperation,
        _event_id: String,
        _client_timestamp: String,
        _seq: i64,
        _payload: serde_json::Value,
    ) -> Result<bool> {
        Ok(false)
    }

    pub async fn apply_remote_events_lww_batch(
        &self,
        _events: Vec<(
            SyncEntity,
            String,
            SyncOperation,
            String,
            String,
            i64,
            serde_json::Value,
        )>,
    ) -> Result<usize> {
        Ok(0)
    }

    pub async fn mark_pull_completed(&self) -> Result<()> {
        Ok(())
    }

    pub async fn mark_cycle_outcome(
        &self,
        _status: String,
        _duration_ms: i64,
        _next_retry_at: Option<String>,
    ) -> Result<()> {
        Ok(())
    }

    pub async fn mark_engine_error(&self, _message: String) -> Result<()> {
        Ok(())
    }

    pub async fn prune_applied_events_up_to_seq(&self, _seq: i64) -> Result<usize> {
        Ok(0)
    }

    // -----------------------------------------------------------------------
    // Snapshot methods (stubs returning errors for unsupported operations)
    // -----------------------------------------------------------------------

    pub async fn export_snapshot_sqlite_image(&self, _tables: Vec<String>) -> Result<Vec<u8>> {
        Err(Error::Database(DatabaseError::Internal(
            "Snapshot export not supported in PostgreSQL mode".to_string(),
        )))
    }

    pub async fn restore_snapshot_tables_from_file(
        &self,
        _snapshot_db_path: String,
        _tables: Vec<String>,
        _cursor_value: i64,
        _device_id: String,
        _key_version: Option<i32>,
    ) -> Result<()> {
        Err(Error::Database(DatabaseError::Internal(
            "Snapshot restore not supported in PostgreSQL mode".to_string(),
        )))
    }
}
