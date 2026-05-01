//! PostgreSQL repository for CSV import templates (D-16/17/18).

use std::sync::Arc;

use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use whaleit_core::errors::Result;
use whaleit_core::transactions::{
    NewTransactionTemplate, TransactionTemplate, TransactionTemplateRepositoryTrait,
    TransactionTemplateUpdate,
};

use crate::db::PgPool;
use crate::errors::{IntoCore, StoragePgError};
use crate::schema::transaction_csv_templates;

use super::templates_model::{TransactionTemplateChangesetDB, TransactionTemplateDB};

pub struct PgTransactionTemplateRepository {
    pool: Arc<PgPool>,
}

impl PgTransactionTemplateRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TransactionTemplateRepositoryTrait for PgTransactionTemplateRepository {
    async fn list_all(&self) -> Result<Vec<TransactionTemplate>> {
        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;
        let rows: Vec<TransactionTemplateDB> = transaction_csv_templates::table
            .order(transaction_csv_templates::name.asc())
            .load::<TransactionTemplateDB>(&mut conn)
            .await
            .into_core()?;
        Ok(rows.into_iter().map(TransactionTemplate::from).collect())
    }

    async fn get_by_id(&self, id: &str) -> Result<TransactionTemplate> {
        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;
        let row: TransactionTemplateDB = transaction_csv_templates::table
            .find(id)
            .first::<TransactionTemplateDB>(&mut conn)
            .await
            .into_core()?;
        Ok(TransactionTemplate::from(row))
    }

    async fn create(&self, new: NewTransactionTemplate) -> Result<TransactionTemplate> {
        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;
        let db: TransactionTemplateDB = new.into();

        // ON CONFLICT (name) DO UPDATE so saving twice with the same name updates
        // the mapping instead of failing (D-17 re-map and save over semantics).
        diesel::sql_query(
            "INSERT INTO transaction_csv_templates (id, name, mapping, header_signature, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6) \
             ON CONFLICT (name) \
             DO UPDATE SET \
               mapping = EXCLUDED.mapping, \
               header_signature = EXCLUDED.header_signature, \
               updated_at = EXCLUDED.updated_at \
             RETURNING id",
        )
        .bind::<diesel::sql_types::Text, _>(&db.id)
        .bind::<diesel::sql_types::Text, _>(&db.name)
        .bind::<diesel::sql_types::Jsonb, _>(&db.mapping)
        .bind::<diesel::sql_types::Array<diesel::sql_types::Nullable<diesel::sql_types::Text>>, _>(&db.header_signature)
        .bind::<diesel::sql_types::Timestamp, _>(db.created_at)
        .bind::<diesel::sql_types::Timestamp, _>(db.updated_at)
        .execute(&mut conn)
        .await
        .into_core()?;

        // Re-fetch by name to get the actual row (handles both insert and update paths).
        let row: TransactionTemplateDB = transaction_csv_templates::table
            .filter(transaction_csv_templates::name.eq(&db.name))
            .first::<TransactionTemplateDB>(&mut conn)
            .await
            .into_core()?;
        Ok(TransactionTemplate::from(row))
    }

    async fn update(&self, update: TransactionTemplateUpdate) -> Result<TransactionTemplate> {
        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;
        let changeset = TransactionTemplateChangesetDB::from(&update);
        diesel::update(transaction_csv_templates::table.find(&update.id))
            .set(&changeset)
            .execute(&mut conn)
            .await
            .into_core()?;
        self.get_by_id(&update.id).await
    }

    async fn delete(&self, id: &str) -> Result<()> {
        let mut conn = self.pool.get().await.map_err(StoragePgError::from)?;
        diesel::delete(transaction_csv_templates::table.find(id))
            .execute(&mut conn)
            .await
            .into_core()?;
        Ok(())
    }
}
