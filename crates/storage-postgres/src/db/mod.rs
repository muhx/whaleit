//! PostgreSQL connection pool and database initialization.
//!
//! Uses deadpool + diesel-async for async connection pooling.
//! No write actor is needed — PostgreSQL handles concurrent writes natively.

use std::sync::Arc;

use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use log::{error, info};

use whaleit_core::errors::{DatabaseError, Result};

const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

/// PostgreSQL connection pool type using deadpool.
pub type PgPool = deadpool::managed::Pool<AsyncDieselConnectionManager<AsyncPgConnection>>;

/// A connection checked out from the deadpool.
pub type PgConnection<'a> =
    deadpool::managed::Object<AsyncDieselConnectionManager<AsyncPgConnection>>;

/// Creates a new PostgreSQL connection pool.
///
/// # Arguments
/// * `database_url` - PostgreSQL connection string (e.g., `postgres://user:pass@host/db`)
/// * `max_size` - Maximum number of connections in the pool (default: 8)
pub fn create_pool(database_url: &str, max_size: usize) -> Result<Arc<PgPool>> {
    let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
    let pool = deadpool::managed::Pool::builder(manager)
        .max_size(max_size)
        .build()
        .map_err(|e: deadpool::managed::BuildError| {
            DatabaseError::PoolCreationFailed(e.to_string())
        })?;
    Ok(Arc::new(pool))
}

/// Tests the database connection and configures PostgreSQL settings.
pub async fn init(database_url: &str) -> Result<()> {
    let mut conn = <AsyncPgConnection as AsyncConnection>::establish(database_url)
        .await
        .map_err(|e| DatabaseError::ConnectionFailed(e.to_string()))?;

    // Verify connectivity
    diesel::sql_query("SELECT 1")
        .execute(&mut conn)
        .await
        .map_err(|e: diesel::result::Error| DatabaseError::ConnectionFailed(e.to_string()))?;

    info!("PostgreSQL connection verified successfully");
    Ok(())
}

/// Runs pending PostgreSQL migrations.
pub async fn run_migrations(database_url: &str) -> Result<()> {
    info!("Running PostgreSQL migrations");

    let conn = <AsyncPgConnection as AsyncConnection>::establish(database_url)
        .await
        .map_err(|e| {
            error!("Failed to connect for migrations: {}", e);
            DatabaseError::ConnectionFailed(e.to_string())
        })?;

    let mut harness = diesel_async::AsyncMigrationHarness::new(conn);
    harness.run_pending_migrations(MIGRATIONS).map_err(
        |e: Box<dyn std::error::Error + Send + Sync>| {
            error!("PostgreSQL migration failed: {}", e);
            DatabaseError::MigrationFailed(e.to_string())
        },
    )?;

    info!("PostgreSQL migrations applied successfully");
    Ok(())
}
