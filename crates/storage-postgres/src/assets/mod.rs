//! PostgreSQL storage implementation for assets.

mod alternative_asset;
mod model;
mod repository;

pub use alternative_asset::PgAlternativeAssetRepository;
pub use model::{AssetDB, InsertableAssetDB};
pub use repository::PgAssetRepository;

// Type alias for compatibility
pub type AlternativeAssetRepository = PgAlternativeAssetRepository;
