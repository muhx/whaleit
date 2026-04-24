//! Health center storage module.

pub mod model;
pub mod repository;

pub use model::HealthIssueDismissalDB;
pub use repository::PgHealthDismissalRepository;
