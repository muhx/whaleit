use async_trait::async_trait;
use std::sync::Arc;

use crate::broker_ingest::ImportRunRepositoryTrait;

/// Bridges connect-owned import-run repository contracts to core activity services.
pub struct CoreImportRunRepositoryAdapter {
    inner: Arc<dyn ImportRunRepositoryTrait>,
}

impl CoreImportRunRepositoryAdapter {
    pub fn new(inner: Arc<dyn ImportRunRepositoryTrait>) -> Self {
        Self { inner }
    }
}

#[async_trait]
impl whaleit_core::activities::ImportRunRepositoryTrait for CoreImportRunRepositoryAdapter {
    async fn create(
        &self,
        import_run: whaleit_core::activities::ImportRun,
    ) -> whaleit_core::errors::Result<whaleit_core::activities::ImportRun> {
        self.inner.create(import_run.into()).await.map(Into::into)
    }

    async fn update(
        &self,
        import_run: whaleit_core::activities::ImportRun,
    ) -> whaleit_core::errors::Result<whaleit_core::activities::ImportRun> {
        self.inner.update(import_run.into()).await.map(Into::into)
    }

    async fn get_by_id(
        &self,
        id: &str,
    ) -> whaleit_core::errors::Result<Option<whaleit_core::activities::ImportRun>> {
        self.inner.get_by_id(id).map(|run| run.map(Into::into))
    }

    async fn get_recent_for_account(
        &self,
        account_id: &str,
        limit: i64,
    ) -> whaleit_core::errors::Result<Vec<whaleit_core::activities::ImportRun>> {
        self.inner
            .get_recent_for_account(account_id, limit)
            .map(|runs| runs.into_iter().map(Into::into).collect())
    }
}
