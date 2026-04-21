//! PostgreSQL storage implementation for assets.

mod model;
mod repository;

pub use model::{AssetDB, InsertableAssetDB};
pub use repository::PgAssetRepository;
