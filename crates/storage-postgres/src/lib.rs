//! PostgreSQL storage implementation for Whaleit.
//!
//! This crate provides all database-related functionality using Diesel ORM with PostgreSQL
//! via diesel-async, using native PostgreSQL features:
//! - Async connection pooling via deadpool + diesel-async
//! - Native UUID primary keys (UUID v7)
//! - Native boolean and timestamptz types
//! - No write actor needed (PG handles concurrent writes natively)
//!
//! # Architecture
//!
//! ```text
//! core (domain)          connect (sync)
//!       │                      │
//!       └──────────┬───────────┘
//!                  │
//!          storage-postgres (this crate)
//!                  │
//!           PostgreSQL DB
//! ```

pub mod db;
pub mod errors;
pub mod schema;

// Repository implementations
pub mod accounts;
pub mod activities;
pub mod ai_chat;
pub mod assets;
pub mod custom_provider;
pub mod fx;
pub mod goals;
pub mod health;
pub mod limits;
pub mod market_data;
pub mod portfolio;
pub mod settings;
pub mod sync;
pub mod taxonomies;
pub mod users;

// Re-export repository types for compatibility
pub use accounts::PgAccountRepository;
pub use activities::PgActivityRepository;
pub use assets::{AlternativeAssetRepository, PgAlternativeAssetRepository, PgAssetRepository};
pub use fx::PgFxRepository;
pub use portfolio::{
    PgSnapshotRepository, PgValuationRepository, SnapshotRepository, ValuationRepository,
};
pub use settings::PgSettingsRepository;
pub use sync::{
    AppSyncRepository, BrokerSyncStateRepository, ImportRunRepository, PlatformRepository,
};
pub use users::PgUserRepository;

// Re-export database utilities
pub use db::{create_pool, init, run_migrations, PgPool};

// Re-export storage errors and conversion helpers
pub use errors::{IntoCore, StoragePgError};

// Re-export from whaleit-core for convenience
pub use whaleit_core::errors::{DatabaseError, Error, Result};
