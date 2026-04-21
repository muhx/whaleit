//! PostgreSQL custom provider repository.

use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use std::sync::Arc;

use super::model::CustomProviderDB;
use crate::db::PgPool;
use crate::errors::StoragePgError;
use crate::schema::market_data_custom_providers;
use whaleit_core::errors::Result;

pub struct PgCustomProviderRepository {
    pool: Arc<PgPool>,
}

impl PgCustomProviderRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

// The custom provider repository doesn't have a standardized trait in core.
// This is a placeholder for future implementation.
