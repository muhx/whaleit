//! PostgreSQL import run repository.

use std::sync::Arc;

use crate::db::PgPool;

pub struct PgImportRunRepository {
    #[allow(dead_code)]
    pool: Arc<PgPool>,
}

impl PgImportRunRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}
