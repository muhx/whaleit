//! PostgreSQL import run repository.
//!
//! Implements both `whaleit_core::activities::ImportRunRepositoryTrait` and
//! `whaleit_connect::ImportRunRepositoryTrait` using diesel-async.

use std::sync::Arc;

use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::db::PgPool;
use crate::errors::IntoCore;
use crate::schema::import_runs;

/// Safely extract the string representation of a serde-serialized enum value.
///
/// This avoids the fragile `serde_json::to_string(...).unwrap_or_default().trim_matches('"')`
/// pattern which can silently produce empty strings or incorrectly strip embedded quotes.
fn enum_to_db_string<T: serde::Serialize>(val: &T) -> String {
    serde_json::to_string(val)
        .ok()
        .and_then(|s| serde_json::from_str::<String>(&s).ok())
        .unwrap_or_default()
}

pub struct PgImportRunRepository {
    pool: Arc<PgPool>,
}

impl PgImportRunRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

/// Implement core's ImportRunRepositoryTrait for activity import tracking
#[async_trait]
impl whaleit_core::activities::ImportRunRepositoryTrait for PgImportRunRepository {
    async fn create(
        &self,
        import_run: whaleit_core::activities::ImportRun,
    ) -> whaleit_core::Result<whaleit_core::activities::ImportRun> {
        let now = chrono::Utc::now().naive_utc();
        let summary_json = import_run
            .summary
            .as_ref()
            .and_then(|s| serde_json::to_string(s).ok());
        let warnings_json = import_run
            .warnings
            .as_ref()
            .and_then(|w| serde_json::to_string(w).ok());

        let mut conn = self.pool.get().await.map_err(|e| {
            whaleit_core::errors::DatabaseError::ConnectionFailed(e.to_string())
        })?;

        diesel::insert_into(import_runs::table)
            .values((
                import_runs::id.eq(&import_run.id),
                import_runs::account_id.eq(&import_run.account_id),
                import_runs::source_system.eq(&import_run.source_system),
                import_runs::run_type.eq(enum_to_db_string(&import_run.run_type)),
                import_runs::mode.eq(enum_to_db_string(&import_run.mode)),
                import_runs::status.eq(enum_to_db_string(&import_run.status)),
                import_runs::started_at.eq(now),
                import_runs::finished_at.eq::<Option<chrono::NaiveDateTime>>(None),
                import_runs::review_mode.eq(enum_to_db_string(&import_run.review_mode)),
                import_runs::applied_at.eq::<Option<chrono::NaiveDateTime>>(None),
                import_runs::checkpoint_in.eq::<Option<String>>(None),
                import_runs::checkpoint_out.eq::<Option<String>>(None),
                import_runs::summary.eq(summary_json),
                import_runs::warnings.eq(warnings_json),
                import_runs::error.eq(&import_run.error),
                import_runs::created_at.eq(now),
                import_runs::updated_at.eq(now),
            ))
            .execute(&mut conn)
            .await
            .into_core()?;

        Ok(import_run)
    }

    async fn update(
        &self,
        import_run: whaleit_core::activities::ImportRun,
    ) -> whaleit_core::Result<whaleit_core::activities::ImportRun> {
        let now = chrono::Utc::now().naive_utc();
        let status_str = enum_to_db_string(&import_run.status);
        let error = import_run.error.clone();
        let summary_json = import_run
            .summary
            .as_ref()
            .and_then(|s| serde_json::to_string(s).ok());
        let warnings_json = import_run
            .warnings
            .as_ref()
            .and_then(|w| serde_json::to_string(w).ok());
        let id = import_run.id.clone();

        let mut conn = self.pool.get().await.map_err(|e| {
            whaleit_core::errors::DatabaseError::ConnectionFailed(e.to_string())
        })?;

        diesel::update(import_runs::table.find(&id))
            .set((
                import_runs::status.eq(&status_str),
                import_runs::error.eq(&error),
                import_runs::summary.eq(&summary_json),
                import_runs::warnings.eq(&warnings_json),
                import_runs::updated_at.eq(now),
            ))
            .execute(&mut conn)
            .await
            .into_core()?;

        Ok(import_run)
    }

    async fn get_by_id(
        &self,
        id: &str,
    ) -> whaleit_core::Result<Option<whaleit_core::activities::ImportRun>> {
        // Not yet needed in PG mode - return None
        Ok(None)
    }

    async fn get_recent_for_account(
        &self,
        _account_id: &str,
        _limit: i64,
    ) -> whaleit_core::Result<Vec<whaleit_core::activities::ImportRun>> {
        Ok(Vec::new())
    }
}

/// Implement connect's ImportRunRepositoryTrait for broker sync tracking
#[async_trait]
impl whaleit_connect::ImportRunRepositoryTrait for PgImportRunRepository {
    async fn create(
        &self,
        import_run: whaleit_connect::ImportRun,
    ) -> whaleit_core::Result<whaleit_connect::ImportRun> {
        // Convert connect ImportRun to DB insert
        let now = chrono::Utc::now().naive_utc();
        let summary_json = import_run
            .summary
            .as_ref()
            .and_then(|s| serde_json::to_string(s).ok());
        let warnings_json = import_run
            .warnings
            .as_ref()
            .and_then(|w| serde_json::to_string(w).ok());
        let checkpoint_in_json = import_run
            .checkpoint_in
            .as_ref()
            .and_then(|c| serde_json::to_string(c).ok());
        let checkpoint_out_json = import_run
            .checkpoint_out
            .as_ref()
            .and_then(|c| serde_json::to_string(c).ok());

        let mut conn = self.pool.get().await.map_err(|e| {
            whaleit_core::errors::DatabaseError::ConnectionFailed(e.to_string())
        })?;

        diesel::insert_into(import_runs::table)
            .values((
                import_runs::id.eq(&import_run.id),
                import_runs::account_id.eq(&import_run.account_id),
                import_runs::source_system.eq(&import_run.source_system),
                import_runs::run_type.eq(enum_to_db_string(&import_run.run_type)),
                import_runs::mode.eq(enum_to_db_string(&import_run.mode)),
                import_runs::status.eq(enum_to_db_string(&import_run.status)),
                import_runs::started_at.eq(now),
                import_runs::finished_at.eq::<Option<chrono::NaiveDateTime>>(None),
                import_runs::review_mode.eq(enum_to_db_string(&import_run.review_mode)),
                import_runs::applied_at.eq::<Option<chrono::NaiveDateTime>>(None),
                import_runs::checkpoint_in.eq(checkpoint_in_json),
                import_runs::checkpoint_out.eq(checkpoint_out_json),
                import_runs::summary.eq(summary_json),
                import_runs::warnings.eq(warnings_json),
                import_runs::error.eq(&import_run.error),
                import_runs::created_at.eq(now),
                import_runs::updated_at.eq(now),
            ))
            .execute(&mut conn)
            .await
            .into_core()?;

        Ok(import_run)
    }

    async fn update(
        &self,
        import_run: whaleit_connect::ImportRun,
    ) -> whaleit_core::Result<whaleit_connect::ImportRun> {
        let now = chrono::Utc::now().naive_utc();
        let status_str = enum_to_db_string(&import_run.status);
        let error = import_run.error.clone();
        let id = import_run.id.clone();

        let mut conn = self.pool.get().await.map_err(|e| {
            whaleit_core::errors::DatabaseError::ConnectionFailed(e.to_string())
        })?;

        diesel::update(import_runs::table.find(&id))
            .set((
                import_runs::status.eq(&status_str),
                import_runs::error.eq(&error),
                import_runs::updated_at.eq(now),
            ))
            .execute(&mut conn)
            .await
            .into_core()?;

        Ok(import_run)
    }

    fn get_by_id(
        &self,
        _id: &str,
    ) -> whaleit_core::Result<Option<whaleit_connect::ImportRun>> {
        // Not yet needed in PG mode
        Ok(None)
    }

    fn get_recent_for_account(
        &self,
        _account_id: &str,
        _limit: i64,
    ) -> whaleit_core::Result<Vec<whaleit_connect::ImportRun>> {
        Ok(Vec::new())
    }

    fn get_all(
        &self,
        _limit: i64,
        _offset: i64,
    ) -> whaleit_core::Result<Vec<whaleit_connect::ImportRun>> {
        Ok(Vec::new())
    }

    fn get_by_run_type(
        &self,
        _run_type: &str,
        _limit: i64,
        _offset: i64,
    ) -> whaleit_core::Result<Vec<whaleit_connect::ImportRun>> {
        Ok(Vec::new())
    }
}
