//! PostgreSQL app sync repository (stub implementation).
//!
//! NOTE: This is a stub implementation for compatibility.
//! Full app sync functionality is not yet implemented for PostgreSQL.

use std::sync::Arc;

use whaleit_core::Result;
use crate::db::PgPool;

pub struct PgAppSyncRepository {
    #[allow(dead_code)]
    pool: Arc<PgPool>,
}

impl PgAppSyncRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    // Stub implementations of sync methods
    // Return errors for complex operations not yet implemented
    pub async fn clear_all_min_snapshot_created_at(&self) -> Result<()> {
        Ok(())
    }

    pub async fn reset_and_mark_bootstrap_complete(&self, _device_id: String, _key_version: String) -> Result<()> {
        Ok(())
    }

    pub async fn reset_local_sync_session(&self) -> Result<()> {
        Ok(())
    }

    pub async fn set_min_snapshot_created_at(&self, _device_id: String, _timestamp: i64) -> Result<()> {
        Ok(())
    }

    pub async fn clear_min_snapshot_created_at(&self, _device_id: String) -> Result<()> {
        Ok(())
    }

    pub async fn upsert_device_config(&self, _config: serde_json::Value) -> Result<()> {
        Ok(())
    }

    pub async fn get_engine_status(&self) -> Result<serde_json::Value> {
        Ok(serde_json::json!({ "status": "idle" }))
    }

    pub async fn needs_bootstrap(&self, _device_id: &str) -> Result<bool> {
        Ok(true)
    }

    pub async fn get_cursor(&self) -> Result<i64> {
        Ok(0)
    }

    pub async fn get_local_sync_data_summary(&self) -> Result<serde_json::Value> {
        Ok(serde_json::json!({ "entities_count": 0, "min_cursor": 0 }))
    }

    pub async fn get_min_snapshot_created_at(&self, _device_id: &str) -> Result<Option<String>> {
        Ok(None)
    }
}
