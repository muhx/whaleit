//! PostgreSQL storage implementation for accounts.

mod model;
mod repository;

pub use model::AccountDB;
pub use repository::PgAccountRepository;

#[cfg(test)]
mod migration_tests;

#[cfg(test)]
mod repository_tests;
