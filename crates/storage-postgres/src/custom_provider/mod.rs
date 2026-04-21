//! PostgreSQL storage implementation for custom providers.

pub mod model;
pub mod repository;

pub use model::CustomProviderDB;
pub use repository::PgCustomProviderRepository;
