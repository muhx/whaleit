//! CSV template repository and service traits (D-16/17/18).

use async_trait::async_trait;

use crate::Result;

use super::templates_model::{
    NewTransactionTemplate, TransactionTemplate, TransactionTemplateUpdate,
};

/// Repository trait for CSV template persistence.
#[async_trait]
pub trait TransactionTemplateRepositoryTrait: Send + Sync {
    async fn list_all(&self) -> Result<Vec<TransactionTemplate>>;
    async fn get_by_id(&self, id: &str) -> Result<TransactionTemplate>;
    async fn create(&self, new: NewTransactionTemplate) -> Result<TransactionTemplate>;
    async fn update(&self, update: TransactionTemplateUpdate) -> Result<TransactionTemplate>;
    async fn delete(&self, id: &str) -> Result<()>;
}

/// Service trait for CSV template orchestration.
#[async_trait]
pub trait TransactionTemplateServiceTrait: Send + Sync {
    async fn list_templates(&self) -> Result<Vec<TransactionTemplate>>;
    async fn get_template(&self, id: &str) -> Result<TransactionTemplate>;
    async fn save_template(&self, new: NewTransactionTemplate) -> Result<TransactionTemplate>;
    async fn update_template(
        &self,
        update: TransactionTemplateUpdate,
    ) -> Result<TransactionTemplate>;
    async fn delete_template(&self, id: &str) -> Result<()>;
}
