//! PostgreSQL contribution limits repository implementation.

use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use std::sync::Arc;

use super::model::{ContributionLimitDB, NewContributionLimitDB};
use crate::db::PgPool;
use crate::errors::StoragePgError;
use crate::schema::contribution_limits;
use crate::schema::contribution_limits::dsl::*;
use whaleit_core::errors::Result;
use whaleit_core::limits::{
    ContributionLimit, ContributionLimitRepositoryTrait, NewContributionLimit,
};

pub struct PgContributionLimitRepository {
    pool: Arc<PgPool>,
}

impl PgContributionLimitRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ContributionLimitRepositoryTrait for PgContributionLimitRepository {
    async fn get_contribution_limit(&self, limit_id: &str) -> Result<ContributionLimit> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        let db = contribution_limits::table
            .find(limit_id)
            .first::<ContributionLimitDB>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;
        Ok(db.into())
    }

    async fn get_contribution_limits(&self) -> Result<Vec<ContributionLimit>> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        let results = contribution_limits::table
            .load::<ContributionLimitDB>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;
        Ok(results.into_iter().map(ContributionLimit::from).collect())
    }

    async fn create_contribution_limit(
        &self,
        new_limit: NewContributionLimit,
    ) -> Result<ContributionLimit> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        let now = chrono::Utc::now().naive_utc();
        let db: NewContributionLimitDB = new_limit.into();

        diesel::insert_into(contribution_limits::table)
            .values((
                contribution_limits::id.eq(&db.id),
                contribution_limits::group_name.eq(&db.group_name),
                contribution_limits::contribution_year.eq(db.contribution_year),
                contribution_limits::limit_amount.eq(db.limit_amount),
                contribution_limits::account_ids.eq(&db.account_ids),
                contribution_limits::start_date.eq(&db.start_date),
                contribution_limits::end_date.eq(&db.end_date),
                contribution_limits::created_at.eq(now),
                contribution_limits::updated_at.eq(now),
            ))
            .execute(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        let result = contribution_limits::table
            .find(&db.id)
            .first::<ContributionLimitDB>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;
        Ok(result.into())
    }

    async fn update_contribution_limit(
        &self,
        limit_id: &str,
        updated_limit: NewContributionLimit,
    ) -> Result<ContributionLimit> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        let now = chrono::Utc::now().naive_utc();

        diesel::update(contribution_limits::table.find(limit_id))
            .set((
                group_name.eq(&updated_limit.group_name),
                contribution_year.eq(updated_limit.contribution_year),
                limit_amount.eq(updated_limit.limit_amount),
                account_ids.eq(&updated_limit.account_ids),
                start_date.eq(&updated_limit.start_date),
                end_date.eq(&updated_limit.end_date),
                updated_at.eq(now),
            ))
            .execute(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        let result = contribution_limits::table
            .find(limit_id)
            .first::<ContributionLimitDB>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;
        Ok(result.into())
    }

    async fn delete_contribution_limit(&self, limit_id: &str) -> Result<()> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        diesel::delete(contribution_limits::table.find(limit_id))
            .execute(&mut conn)
            .await
            .map_err(StoragePgError::from)?;
        Ok(())
    }
}
