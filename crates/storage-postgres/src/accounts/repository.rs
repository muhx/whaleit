//! PostgreSQL accounts repository implementation.

use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use std::sync::Arc;
use uuid::Uuid;

use super::model::AccountDB;
use crate::db::PgPool;
use crate::errors::StoragePgError;
use crate::schema::accounts;
use crate::schema::accounts::dsl::*;
use whaleit_core::accounts::{Account, AccountRepositoryTrait, AccountUpdate, NewAccount};
use whaleit_core::errors::Result;

pub struct PgAccountRepository {
    pool: Arc<PgPool>,
}

impl PgAccountRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AccountRepositoryTrait for PgAccountRepository {
    async fn create(&self, new_account: NewAccount) -> Result<Account> {
        new_account.validate()?;
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;

        let mut account_db: AccountDB = new_account.into();
        account_db.id = Uuid::now_v7().to_string();

        diesel::insert_into(accounts::table)
            .values(&account_db)
            .execute(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        Ok(account_db.into())
    }

    async fn update(&self, account_update: AccountUpdate) -> Result<Account> {
        account_update.validate()?;
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;

        let is_archived_provided = account_update.is_archived.is_some();
        let tracking_mode_provided = account_update.tracking_mode.is_some();

        let mut account_db: AccountDB = account_update.into();

        let existing = accounts::table
            .select(AccountDB::as_select())
            .find(&account_db.id)
            .first::<AccountDB>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        account_db.currency = existing.currency;
        account_db.created_at = existing.created_at;
        account_db.updated_at = chrono::Utc::now().naive_utc();
        account_db.provider_account_id = existing.provider_account_id;
        account_db.platform_id = existing.platform_id;
        account_db.provider = existing.provider;
        account_db.account_number = existing.account_number;
        account_db.meta = existing.meta;

        if !is_archived_provided {
            account_db.is_archived = existing.is_archived;
        }
        if !tracking_mode_provided {
            account_db.tracking_mode = existing.tracking_mode;
        }

        diesel::update(accounts::table.find(&account_db.id))
            .set(&account_db)
            .execute(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        Ok(account_db.into())
    }

    async fn get_by_id(&self, account_id_param: &str) -> Result<Account> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        let account = accounts::table
            .select(AccountDB::as_select())
            .find(account_id_param)
            .first::<AccountDB>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;
        Ok(account.into())
    }

    async fn list(
        &self,
        is_active_filter: Option<bool>,
        is_archived_filter: Option<bool>,
        account_ids_filter: Option<&[String]>,
    ) -> Result<Vec<Account>> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;

        let mut query = accounts::table.into_boxed();

        if let Some(active) = is_active_filter {
            query = query.filter(is_active.eq(active));
        }

        if let Some(archived) = is_archived_filter {
            query = query.filter(is_archived.eq(archived));
        }

        if let Some(ids) = account_ids_filter {
            query = query.filter(id.eq_any(ids));
        }

        let results = query
            .select(AccountDB::as_select())
            .order((is_active.desc(), is_archived.asc(), name.asc()))
            .load::<AccountDB>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        Ok(results.into_iter().map(Account::from).collect())
    }

    async fn delete(&self, account_id_param: &str) -> Result<usize> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        let affected_rows = diesel::delete(accounts::table.find(account_id_param))
            .execute(&mut conn)
            .await
            .map_err(StoragePgError::from)?;
        Ok(affected_rows)
    }
}
