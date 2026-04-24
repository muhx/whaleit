//! PostgreSQL storage implementation for portfolio data.

pub mod snapshot;
pub mod valuation;

// Re-export PostgreSQL repository implementations
pub use snapshot::PgSnapshotRepository;
pub use valuation::PgValuationRepository;

// Type aliases for compatibility with SQLite storage API
pub type SnapshotRepository = PgSnapshotRepository;
pub type ValuationRepository = PgValuationRepository;
