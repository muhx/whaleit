//! PostgreSQL AI chat repository.

use async_trait::async_trait;
use std::sync::Arc;

use crate::db::PgPool;
use whaleit_core::errors::Result;

pub struct PgAiChatRepository {
    #[allow(dead_code)]
    pool: Arc<PgPool>,
}

impl PgAiChatRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

// AI chat repository doesn't have a standardized trait in core yet.
// This is a placeholder for future implementation.
