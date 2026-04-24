use crate::errors::Result;
use async_trait::async_trait;
use chrono::NaiveDateTime;

use super::model::{ApiKey, User, VerificationToken};

#[async_trait]
pub trait UserRepositoryTrait: Send + Sync {
    async fn create_user(
        &self,
        email: &str,
        password_hash: &str,
        display_name: Option<&str>,
    ) -> Result<User>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>>;
    async fn find_by_id(&self, id: &str) -> Result<Option<User>>;
    async fn verify_email(&self, user_id: &str) -> Result<()>;
    async fn update_password(&self, user_id: &str, new_hash: &str) -> Result<()>;
    async fn create_token(
        &self,
        user_id: &str,
        token_hash: &str,
        token_type: &str,
        expires_at: NaiveDateTime,
    ) -> Result<()>;
    async fn find_valid_token(
        &self,
        token_hash: &str,
        token_type: &str,
    ) -> Result<Option<VerificationToken>>;
    async fn consume_token(&self, token_id: &str) -> Result<()>;
    async fn create_api_key(
        &self,
        user_id: &str,
        key_prefix: &str,
        key_hash: &str,
        name: &str,
        expires_at: Option<NaiveDateTime>,
    ) -> Result<ApiKey>;
    async fn find_api_key_by_hash(&self, key_hash: &str) -> Result<Option<ApiKey>>;
    async fn list_api_keys(&self, user_id: &str) -> Result<Vec<ApiKey>>;
    async fn delete_api_key(&self, id: &str, user_id: &str) -> Result<()>;
    async fn update_api_key_last_used(&self, id: &str) -> Result<()>;
}
