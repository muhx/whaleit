//! PostgreSQL app sync repository.

use std::sync::Arc;

use crate::db::PgPool;

pub struct PgAppSyncRepository {
    #[allow(dead_code)]
    pool: Arc<PgPool>,
}

impl PgAppSyncRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}
