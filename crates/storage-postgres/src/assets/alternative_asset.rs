//! PostgreSQL storage implementation for alternative assets.
//!
//! NOTE: This is a stub implementation for compatibility.
//! Alternative assets feature is not yet fully implemented for PostgreSQL.

use whaleit_core::errors::{DatabaseError, Error, Result};

/// PostgreSQL repository for alternative assets (stub implementation).
pub struct PgAlternativeAssetRepository {
    _pool: std::sync::Arc<crate::db::PgPool>,
}

impl PgAlternativeAssetRepository {
    pub fn new(pool: std::sync::Arc<crate::db::PgPool>) -> Self {
        Self { _pool: pool }
    }
}

#[async_trait::async_trait]
impl whaleit_core::assets::AlternativeAssetRepositoryTrait for PgAlternativeAssetRepository {
    async fn delete_alternative_asset(&self, _asset_id: &str) -> Result<()> {
        Err(Error::Database(DatabaseError::Internal(
            "Alternative assets are not yet supported in PostgreSQL mode".to_string(),
        )))
    }

    async fn update_asset_metadata(
        &self,
        _asset_id: &str,
        _metadata: Option<serde_json::Value>,
    ) -> Result<()> {
        Err(Error::Database(DatabaseError::Internal(
            "Alternative assets are not yet supported in PostgreSQL mode".to_string(),
        )))
    }

    async fn find_liabilities_linked_to(&self, _linked_asset_id: &str) -> Result<Vec<String>> {
        Err(Error::Database(DatabaseError::Internal(
            "Alternative assets are not yet supported in PostgreSQL mode".to_string(),
        )))
    }

    async fn update_asset_details(
        &self,
        _asset_id: &str,
        _name: Option<&str>,
        _display_code: Option<&str>,
        _metadata: Option<serde_json::Value>,
        _notes: Option<&str>,
    ) -> Result<()> {
        Err(Error::Database(DatabaseError::Internal(
            "Alternative assets are not yet supported in PostgreSQL mode".to_string(),
        )))
    }
}
