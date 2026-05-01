//! Transactions storage (Phase 4) — model + repository + tests.

pub mod model;
pub mod repository;
pub mod templates_model;
pub mod templates_repository;

pub use model::{PayeeCategoryMemoryDB, TransactionDB, TransactionSplitDB};
pub use repository::PgTransactionRepository;
pub use templates_model::TransactionTemplateDB;
pub use templates_repository::PgTransactionTemplateRepository;

#[cfg(test)]
mod migration_tests;
#[cfg(test)]
mod repository_tests;
