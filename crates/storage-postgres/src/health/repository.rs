//! PostgreSQL health dismissal repository implementation.

use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use std::sync::Arc;

use super::model::HealthIssueDismissalDB;
use crate::db::PgPool;
use crate::errors::StoragePgError;
use crate::schema::health_issue_dismissals;
use crate::schema::health_issue_dismissals::dsl::*;
use whaleit_core::health::{HealthDismissalStore, IssueDismissal};
use whaleit_core::Result;

pub struct PgHealthDismissalRepository {
    pool: Arc<PgPool>,
}

impl PgHealthDismissalRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl HealthDismissalStore for PgHealthDismissalRepository {
    async fn save_dismissal(&self, dismissal: &IssueDismissal) -> Result<()> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        let dismissal_db: HealthIssueDismissalDB = dismissal.clone().into();

        diesel::insert_into(health_issue_dismissals::table)
            .values(&dismissal_db)
            .on_conflict(issue_id)
            .do_update()
            .set(&dismissal_db)
            .execute(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        Ok(())
    }

    async fn remove_dismissal(&self, id: &str) -> Result<()> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        diesel::delete(health_issue_dismissals::table.find(id))
            .execute(&mut conn)
            .await
            .map_err(StoragePgError::from)?;
        Ok(())
    }

    async fn get_dismissals(&self) -> Result<Vec<IssueDismissal>> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        let dismissals_db = health_issue_dismissals::table
            .load::<HealthIssueDismissalDB>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;
        Ok(dismissals_db
            .into_iter()
            .map(IssueDismissal::from)
            .collect())
    }

    async fn get_dismissal(&self, id: &str) -> Result<Option<IssueDismissal>> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        let result = health_issue_dismissals::table
            .find(id)
            .first::<HealthIssueDismissalDB>(&mut conn)
            .await
            .optional()
            .map_err(StoragePgError::from)?;
        Ok(result.map(IssueDismissal::from))
    }

    async fn clear_all(&self) -> Result<()> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        diesel::delete(health_issue_dismissals::table)
            .execute(&mut conn)
            .await
            .map_err(StoragePgError::from)?;
        Ok(())
    }
}
