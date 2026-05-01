//! Transactions module - domain models, services, traits, parsers (Phase 4).

mod compiler;
mod csv_parser;
mod duplicate_detector;
mod idempotency;
mod merchant_normalizer;
mod ofx_parser;
mod reconciliation;
mod templates_model;
mod templates_service;
mod templates_traits;
mod transactions_constants;
mod transactions_errors;
mod transactions_model;
mod transactions_service;
mod transactions_traits;

#[cfg(test)]
mod duplicate_detector_tests;
#[cfg(test)]
mod merchant_normalizer_tests;
#[cfg(test)]
mod ofx_parser_tests;
#[cfg(test)]
mod reconciliation_tests;
#[cfg(test)]
mod transactions_service_tests;

pub use duplicate_detector::{DuplicateBucket, DuplicateMatch};
pub use templates_model::{NewTransactionTemplate, TransactionTemplate, TransactionTemplateUpdate};
pub use templates_service::TransactionTemplateService;
pub use templates_traits::{TransactionTemplateRepositoryTrait, TransactionTemplateServiceTrait};
pub use transactions_constants::*;
pub use transactions_errors::TransactionError;
pub use transactions_model::{
    NewSplit, NewTransaction, PayeeCategoryMemory, SplitUpdate, Transaction, TransactionSplit,
    TransactionUpdate,
};
pub use transactions_service::TransactionService;
pub use transactions_traits::{
    CsvImportRequest, ImportResult, NewTransferLeg, OfxImportRequest,
    PayeeCategoryMemoryRepositoryTrait, TransactionFilters, TransactionRepositoryTrait,
    TransactionSearchResult, TransactionServiceTrait, TransactionWithRunningBalance,
    TransferEditMode,
};
