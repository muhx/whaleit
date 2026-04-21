//! PostgreSQL broker sync state repository.

use std::sync::Arc;

use crate::db::PgPool;

pub struct PgBrokerSyncStateRepository {
    #[allow(dead_code)]
    pool: Arc<PgPool>,
}

impl PgBrokerSyncStateRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}
