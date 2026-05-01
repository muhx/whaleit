//! Transaction core domain — models, traits, and types.

pub mod templates_model;
pub mod templates_traits;
pub mod transactions_model;
pub mod transactions_traits;

pub use templates_model::{
    CsvFieldMapping, NewTransactionTemplate, TransactionTemplate, TransactionTemplateUpdate,
};
pub use templates_traits::TransactionTemplateRepositoryTrait;
pub use transactions_model::{
    ImportResult, NewSplit, NewTransaction, PayeeCategoryMemory, Transaction, TransactionFilters,
    TransactionSearchResult, TransactionSplit, TransactionUpdate, TransactionWithRunningBalance,
};
pub use transactions_traits::{PayeeCategoryMemoryRepositoryTrait, TransactionRepositoryTrait};
