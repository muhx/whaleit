//! PostgreSQL platform repository.

use std::sync::Arc;

use crate::db::PgPool;

pub struct PgPlatformRepository {
    #[allow(dead_code)]
    pool: Arc<PgPool>,
}

impl PgPlatformRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}
