use async_trait::async_trait;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use std::sync::Arc;
use uuid::Uuid;
use whaleit_core::errors::{DatabaseError, Result};
use whaleit_core::users::UserRepositoryTrait;

use super::model::*;
use crate::db::PgPool;
use crate::errors::StoragePgError;
use crate::schema::{api_keys, users, verification_tokens};

pub struct PgUserRepository {
    pool: Arc<PgPool>,
}

impl PgUserRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        PgUserRepository { pool }
    }
}

#[async_trait]
impl UserRepositoryTrait for PgUserRepository {
    async fn create_user(
        &self,
        email: &str,
        password_hash: &str,
        display_name: Option<&str>,
    ) -> Result<whaleit_core::users::User> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        let id = Uuid::now_v7().to_string();
        let new_user = NewUserDB {
            id,
            email: email.to_string(),
            password_hash: password_hash.to_string(),
            display_name: display_name.map(|s| s.to_string()),
        };
        Ok(diesel::insert_into(users::table)
            .values(&new_user)
            .get_result::<UserDB>(&mut conn)
            .await
            .map_err(|e| {
                if let diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::UniqueViolation,
                    _,
                ) = &e
                {
                    whaleit_core::errors::Error::Database(DatabaseError::UniqueViolation(format!(
                        "Email already registered: {email}"
                    )))
                } else {
                    StoragePgError::from(e).into()
                }
            })?
            .into())
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<whaleit_core::users::User>> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        users::table
            .filter(users::email.eq(email))
            .first::<UserDB>(&mut conn)
            .await
            .optional()
            .map_err(StoragePgError::from)
            .map(|opt| opt.map(Into::into))
            .map_err(Into::into)
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<whaleit_core::users::User>> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        users::table
            .filter(users::id.eq(id))
            .first::<UserDB>(&mut conn)
            .await
            .optional()
            .map_err(StoragePgError::from)
            .map(|opt| opt.map(Into::into))
            .map_err(Into::into)
    }

    async fn verify_email(&self, user_id: &str) -> Result<()> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        diesel::update(users::table.filter(users::id.eq(user_id)))
            .set((
                users::email_verified.eq(true),
                users::updated_at.eq(diesel::dsl::now),
            ))
            .execute(&mut conn)
            .await
            .map_err(StoragePgError::from)?;
        Ok(())
    }

    async fn update_password(&self, user_id: &str, new_hash: &str) -> Result<()> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        diesel::update(users::table.filter(users::id.eq(user_id)))
            .set((
                users::password_hash.eq(new_hash),
                users::updated_at.eq(diesel::dsl::now),
            ))
            .execute(&mut conn)
            .await
            .map_err(StoragePgError::from)?;
        Ok(())
    }

    async fn create_token(
        &self,
        user_id: &str,
        token_hash: &str,
        token_type: &str,
        expires_at: NaiveDateTime,
    ) -> Result<()> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        let new_token = NewVerificationTokenDB {
            id: Uuid::now_v7().to_string(),
            user_id: user_id.to_string(),
            token_hash: token_hash.to_string(),
            token_type: token_type.to_string(),
            expires_at,
        };
        diesel::insert_into(verification_tokens::table)
            .values(&new_token)
            .execute(&mut conn)
            .await
            .map_err(StoragePgError::from)?;
        Ok(())
    }

    async fn find_valid_token(
        &self,
        token_hash: &str,
        token_type: &str,
    ) -> Result<Option<whaleit_core::users::VerificationToken>> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        verification_tokens::table
            .filter(verification_tokens::token_hash.eq(token_hash))
            .filter(verification_tokens::token_type.eq(token_type))
            .filter(verification_tokens::used_at.is_null())
            .filter(verification_tokens::expires_at.gt(diesel::dsl::now))
            .first::<VerificationTokenDB>(&mut conn)
            .await
            .optional()
            .map_err(StoragePgError::from)
            .map(|opt| opt.map(Into::into))
            .map_err(Into::into)
    }

    async fn consume_token(&self, token_id: &str) -> Result<()> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        diesel::update(verification_tokens::table.filter(verification_tokens::id.eq(token_id)))
            .set(verification_tokens::used_at.eq(diesel::dsl::now))
            .execute(&mut conn)
            .await
            .map_err(StoragePgError::from)?;
        Ok(())
    }

    async fn create_api_key(
        &self,
        user_id: &str,
        key_prefix: &str,
        key_hash: &str,
        name: &str,
        expires_at: Option<NaiveDateTime>,
    ) -> Result<whaleit_core::users::ApiKey> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        let new_key = NewApiKeyDB {
            id: Uuid::now_v7().to_string(),
            user_id: user_id.to_string(),
            key_prefix: key_prefix.to_string(),
            key_hash: key_hash.to_string(),
            name: name.to_string(),
            expires_at,
        };
        diesel::insert_into(api_keys::table)
            .values(&new_key)
            .get_result::<ApiKeyDB>(&mut conn)
            .await
            .map_err(StoragePgError::from)
            .map(|db| db.into())
            .map_err(Into::into)
    }

    async fn find_api_key_by_hash(
        &self,
        key_hash: &str,
    ) -> Result<Option<whaleit_core::users::ApiKey>> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        api_keys::table
            .filter(api_keys::key_hash.eq(key_hash))
            .filter(api_keys::is_active.eq(true))
            .first::<ApiKeyDB>(&mut conn)
            .await
            .optional()
            .map_err(StoragePgError::from)
            .map(|opt| opt.map(Into::into))
            .map_err(Into::into)
    }

    async fn list_api_keys(&self, user_id: &str) -> Result<Vec<whaleit_core::users::ApiKey>> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        api_keys::table
            .filter(api_keys::user_id.eq(user_id))
            .filter(api_keys::is_active.eq(true))
            .order(api_keys::created_at.desc())
            .load::<ApiKeyDB>(&mut conn)
            .await
            .map_err(StoragePgError::from)
            .map(|rows| rows.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    async fn delete_api_key(&self, id: &str, user_id: &str) -> Result<()> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        let count = diesel::update(
            api_keys::table
                .filter(api_keys::id.eq(id))
                .filter(api_keys::user_id.eq(user_id)),
        )
        .set(api_keys::is_active.eq(false))
        .execute(&mut conn)
        .await
        .map_err(StoragePgError::from)?;
        if count == 0 {
            return Err(DatabaseError::NotFound("API key not found".to_string()).into());
        }
        Ok(())
    }

    async fn update_api_key_last_used(&self, id: &str) -> Result<()> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        diesel::update(api_keys::table.filter(api_keys::id.eq(id)))
            .set(api_keys::last_used_at.eq(diesel::dsl::now))
            .execute(&mut conn)
            .await
            .map_err(StoragePgError::from)?;
        Ok(())
    }
}
