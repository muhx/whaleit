//! Transaction template service implementation (Phase 4, plan 04-04).

use std::sync::Arc;

use async_trait::async_trait;

use crate::Result;

use super::{
    templates_model::{NewTransactionTemplate, TransactionTemplate, TransactionTemplateUpdate},
    templates_traits::{TransactionTemplateRepositoryTrait, TransactionTemplateServiceTrait},
};

/// Concrete CSV template service.
pub struct TransactionTemplateService {
    repo: Arc<dyn TransactionTemplateRepositoryTrait>,
}

impl TransactionTemplateService {
    pub fn new(repo: Arc<dyn TransactionTemplateRepositoryTrait>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl TransactionTemplateServiceTrait for TransactionTemplateService {
    async fn list_templates(&self) -> Result<Vec<TransactionTemplate>> {
        self.repo.list_all().await
    }

    async fn get_template(&self, id: &str) -> Result<TransactionTemplate> {
        self.repo.get_by_id(id).await
    }

    async fn save_template(&self, new: NewTransactionTemplate) -> Result<TransactionTemplate> {
        self.repo.create(new).await
    }

    async fn update_template(
        &self,
        update: TransactionTemplateUpdate,
    ) -> Result<TransactionTemplate> {
        self.repo.update(update).await
    }

    async fn delete_template(&self, id: &str) -> Result<()> {
        self.repo.delete(id).await
    }
}
