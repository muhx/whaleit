//! PostgreSQL repository for app sync state (stub implementation).
//!
//! NOTE: This is a stub implementation for compatibility.
//! Full app sync functionality is not yet implemented for PostgreSQL.

use whaleit_core::Result;

/// PostgreSQL repository for app sync (stub implementation).
pub struct PgAppSyncRepository {
    _pool: deadpool::managed::Pool<
        diesel_async::pooled_connection::AsyncDieselConnectionManager<diesel_async::AsyncPgConnection>,
    >,
}

impl PgAppSyncRepository {
    pub fn new(
        pool: deadpool::managed::Pool<
            diesel_async::pooled_connection::AsyncDieselConnectionManager<diesel_async::AsyncPgConnection>,
        >,
    ) -> Self {
        Self { _pool: pool }
    }
}

// Stub implementations of sync methods
impl PgAppSyncRepository {
    pub async fn clear_all_min_snapshot_created_at(&self) -> Result<()> {
        // Stub: app sync not yet fully implemented for PostgreSQL
        Ok(())
    }

    pub async fn reset_and_mark_bootstrap_complete(&self) -> Result<()> {
        // Stub: app sync not yet fully implemented for PostgreSQL
        Ok(())
    }

    pub async fn reset_local_sync_session(&self) -> Result<()> {
        // Stub: app sync not yet fully implemented for PostgreSQL
        Ok(())
    }

    pub async fn set_min_snapshot_created_at(&self, _timestamp: i64) -> Result<()> {
        // Stub: app sync not yet fully implemented for PostgreSQL
        Ok(())
    }
}
