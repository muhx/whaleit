//! PostgreSQL broker sync state repository.
//!
//! Implements `BrokerSyncStateRepositoryTrait` from `whaleit-connect`.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use std::sync::Arc;

use crate::db::PgPool;
use crate::errors::IntoCore;
use crate::schema::brokers_sync_state;

use whaleit_connect::broker_ingest::{
    BrokerSyncState, BrokerSyncStateRepositoryTrait, SyncStatus,
};
use whaleit_core::errors::Result;

#[derive(Queryable, Identifiable, Selectable, PartialEq, Debug, Clone)]
#[diesel(primary_key(account_id, provider))]
#[diesel(table_name = brokers_sync_state)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct BrokerSyncStateDB {
    account_id: String,
    provider: String,
    checkpoint_json: Option<String>,
    last_attempted_at: Option<chrono::NaiveDateTime>,
    last_successful_at: Option<chrono::NaiveDateTime>,
    last_error: Option<String>,
    last_run_id: Option<String>,
    sync_status: String,
    created_at: chrono::NaiveDateTime,
    updated_at: chrono::NaiveDateTime,
}

impl From<BrokerSyncStateDB> for BrokerSyncState {
    fn from(db: BrokerSyncStateDB) -> Self {
        let sync_status = match db.sync_status.as_str() {
            "IDLE" => SyncStatus::Idle,
            "RUNNING" | "SYNCING" => SyncStatus::Running,
            "NEEDS_REVIEW" => SyncStatus::NeedsReview,
            "FAILED" => SyncStatus::Failed,
            _ => SyncStatus::Idle,
        };

        Self {
            account_id: db.account_id,
            provider: db.provider,
            checkpoint_json: db
                .checkpoint_json
                .and_then(|s| serde_json::from_str(&s).ok()),
            last_attempted_at: db.last_attempted_at.map(|ndt| {
                DateTime::from_naive_utc_and_offset(ndt, Utc)
            }),
            last_successful_at: db.last_successful_at.map(|ndt| {
                DateTime::from_naive_utc_and_offset(ndt, Utc)
            }),
            last_error: db.last_error,
            last_run_id: db.last_run_id,
            sync_status,
            created_at: DateTime::from_naive_utc_and_offset(db.created_at, Utc),
            updated_at: DateTime::from_naive_utc_and_offset(db.updated_at, Utc),
        }
    }
}

pub struct PgBrokerSyncStateRepository {
    pool: Arc<PgPool>,
}

impl PgBrokerSyncStateRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl BrokerSyncStateRepositoryTrait for PgBrokerSyncStateRepository {
    fn get_by_account_id(&self, account_id: &str) -> Result<Option<BrokerSyncState>> {
        let pool = self.pool.clone();
        let account_id = account_id.to_string();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let mut conn = pool.get().await.map_err(|e| {
                    whaleit_core::errors::DatabaseError::ConnectionFailed(e.to_string())
                })?;

                let results = brokers_sync_state::table
                    .filter(brokers_sync_state::account_id.eq(&account_id))
                    .load::<BrokerSyncStateDB>(&mut conn)
                    .await
                    .into_core()?;

                Ok(results.into_iter().map(BrokerSyncState::from).next())
            })
        })
    }

    async fn upsert_attempt(&self, account_id: String, provider: String) -> Result<()> {
        let now = chrono::Utc::now().naive_utc();

        let mut conn = self.pool.get().await.map_err(|e| {
            whaleit_core::errors::DatabaseError::ConnectionFailed(e.to_string())
        })?;

        diesel::insert_into(brokers_sync_state::table)
            .values((
                brokers_sync_state::account_id.eq(&account_id),
                brokers_sync_state::provider.eq(&provider),
                brokers_sync_state::sync_status.eq("RUNNING"),
                brokers_sync_state::last_attempted_at.eq(now),
                brokers_sync_state::updated_at.eq(now),
            ))
            .on_conflict((brokers_sync_state::account_id, brokers_sync_state::provider))
            .do_update()
            .set((
                brokers_sync_state::sync_status.eq("RUNNING"),
                brokers_sync_state::last_attempted_at.eq(now),
                brokers_sync_state::updated_at.eq(now),
            ))
            .execute(&mut conn)
            .await
            .into_core()?;

        Ok(())
    }

    async fn upsert_success(
        &self,
        account_id: String,
        provider: String,
        last_synced_date: String,
        import_run_id: Option<String>,
    ) -> Result<()> {
        let now = chrono::Utc::now().naive_utc();

        let mut conn = self.pool.get().await.map_err(|e| {
            whaleit_core::errors::DatabaseError::ConnectionFailed(e.to_string())
        })?;

        diesel::insert_into(brokers_sync_state::table)
            .values((
                brokers_sync_state::account_id.eq(&account_id),
                brokers_sync_state::provider.eq(&provider),
                brokers_sync_state::sync_status.eq("IDLE"),
                brokers_sync_state::last_attempted_at.eq(now),
                brokers_sync_state::last_successful_at.eq(now),
                brokers_sync_state::last_run_id.eq(&import_run_id),
                brokers_sync_state::updated_at.eq(now),
            ))
            .on_conflict((brokers_sync_state::account_id, brokers_sync_state::provider))
            .do_update()
            .set((
                brokers_sync_state::sync_status.eq("IDLE"),
                brokers_sync_state::last_attempted_at.eq(now),
                brokers_sync_state::last_successful_at.eq(now),
                brokers_sync_state::last_run_id.eq(&import_run_id),
                brokers_sync_state::last_error.eq::<Option<String>>(None),
                brokers_sync_state::updated_at.eq(now),
            ))
            .execute(&mut conn)
            .await
            .into_core()?;

        Ok(())
    }

    async fn upsert_failure(
        &self,
        account_id: String,
        provider: String,
        error: String,
        import_run_id: Option<String>,
    ) -> Result<()> {
        let now = chrono::Utc::now().naive_utc();

        let mut conn = self.pool.get().await.map_err(|e| {
            whaleit_core::errors::DatabaseError::ConnectionFailed(e.to_string())
        })?;

        diesel::insert_into(brokers_sync_state::table)
            .values((
                brokers_sync_state::account_id.eq(&account_id),
                brokers_sync_state::provider.eq(&provider),
                brokers_sync_state::sync_status.eq("FAILED"),
                brokers_sync_state::last_attempted_at.eq(now),
                brokers_sync_state::last_error.eq(&error),
                brokers_sync_state::last_run_id.eq(&import_run_id),
                brokers_sync_state::updated_at.eq(now),
            ))
            .on_conflict((brokers_sync_state::account_id, brokers_sync_state::provider))
            .do_update()
            .set((
                brokers_sync_state::sync_status.eq("FAILED"),
                brokers_sync_state::last_attempted_at.eq(now),
                brokers_sync_state::last_error.eq(&error),
                brokers_sync_state::last_run_id.eq(&import_run_id),
                brokers_sync_state::updated_at.eq(now),
            ))
            .execute(&mut conn)
            .await
            .into_core()?;

        Ok(())
    }

    async fn upsert_needs_review(
        &self,
        account_id: String,
        provider: String,
        warning: String,
        import_run_id: Option<String>,
    ) -> Result<()> {
        let now = chrono::Utc::now().naive_utc();

        let mut conn = self.pool.get().await.map_err(|e| {
            whaleit_core::errors::DatabaseError::ConnectionFailed(e.to_string())
        })?;

        diesel::insert_into(brokers_sync_state::table)
            .values((
                brokers_sync_state::account_id.eq(&account_id),
                brokers_sync_state::provider.eq(&provider),
                brokers_sync_state::sync_status.eq("NEEDS_REVIEW"),
                brokers_sync_state::last_attempted_at.eq(now),
                brokers_sync_state::last_error.eq(&warning),
                brokers_sync_state::last_run_id.eq(&import_run_id),
                brokers_sync_state::updated_at.eq(now),
            ))
            .on_conflict((brokers_sync_state::account_id, brokers_sync_state::provider))
            .do_update()
            .set((
                brokers_sync_state::sync_status.eq("NEEDS_REVIEW"),
                brokers_sync_state::last_attempted_at.eq(now),
                brokers_sync_state::last_error.eq(&warning),
                brokers_sync_state::last_run_id.eq(&import_run_id),
                brokers_sync_state::updated_at.eq(now),
            ))
            .execute(&mut conn)
            .await
            .into_core()?;

        Ok(())
    }

    fn get_all(&self) -> Result<Vec<BrokerSyncState>> {
        let pool = self.pool.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let mut conn = pool.get().await.map_err(|e| {
                    whaleit_core::errors::DatabaseError::ConnectionFailed(e.to_string())
                })?;

                let results = brokers_sync_state::table
                    .load::<BrokerSyncStateDB>(&mut conn)
                    .await
                    .into_core()?;

                Ok(results.into_iter().map(BrokerSyncState::from).collect())
            })
        })
    }
}
