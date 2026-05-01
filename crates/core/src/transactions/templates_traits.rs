//! CSV import template repository trait (D-16/17/18).

use async_trait::async_trait;

use super::templates_model::{
    NewTransactionTemplate, TransactionTemplate, TransactionTemplateUpdate,
};
use crate::errors::Result;

/// Repository trait for user-saved CSV import templates.
#[async_trait]
pub trait TransactionTemplateRepositoryTrait: Send + Sync {
    /// Lists all templates ordered by name ASC.
    async fn list_all(&self) -> Result<Vec<TransactionTemplate>>;

    /// Gets a template by ID.
    async fn get_by_id(&self, id: &str) -> Result<TransactionTemplate>;

    /// Creates a new template. On conflict by name, updates the existing row.
    async fn create(&self, new: NewTransactionTemplate) -> Result<TransactionTemplate>;

    /// Updates an existing template.
    async fn update(&self, update: TransactionTemplateUpdate) -> Result<TransactionTemplate>;

    /// Deletes a template by ID.
    async fn delete(&self, id: &str) -> Result<()>;
}
