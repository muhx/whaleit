//! PostgreSQL storage implementation for FX/currency.

pub(crate) mod model;
mod repository;

pub use model::{NewQuoteDB, QuoteDB};
pub use repository::PgFxRepository;
