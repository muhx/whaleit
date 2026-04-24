//! Storage-specific error types for PostgreSQL operations.
//!
//! This module provides error types that wrap diesel-async/PostgreSQL-specific errors
//! and convert them to the database-agnostic error types defined in `whaleit_core`.

use diesel::result::Error as DieselError;
use thiserror::Error;
use whaleit_core::errors::{DatabaseError, Error};

/// Storage-specific errors that wrap Diesel and deadpool types.
///
/// These errors are internal to the storage layer and are converted to
/// `whaleit_core::Error` before being returned to callers.
#[derive(Error, Debug)]
pub enum StoragePgError {
    #[error("PostgreSQL connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Connection pool error: {0}")]
    PoolError(String),

    #[error("Query execution failed: {0}")]
    QueryFailed(#[from] DieselError),

    #[error("Migration failed: {0}")]
    MigrationFailed(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Core error: {0}")]
    CoreError(String),
}

/// Convert core Error to StoragePgError (for transaction wrapper)
impl From<Error> for StoragePgError {
    fn from(err: Error) -> Self {
        StoragePgError::CoreError(err.to_string())
    }
}

impl<E: std::fmt::Debug> From<deadpool::managed::PoolError<E>> for StoragePgError
where
    E: std::fmt::Display,
{
    fn from(err: deadpool::managed::PoolError<E>) -> Self {
        StoragePgError::PoolError(err.to_string())
    }
}

impl From<StoragePgError> for Error {
    fn from(err: StoragePgError) -> Self {
        match err {
            StoragePgError::ConnectionFailed(e) => {
                Error::Database(DatabaseError::ConnectionFailed(e))
            }
            StoragePgError::PoolError(e) => Error::Database(DatabaseError::PoolCreationFailed(e)),
            StoragePgError::QueryFailed(DieselError::NotFound) => {
                Error::Database(DatabaseError::NotFound("Record not found".to_string()))
            }
            StoragePgError::QueryFailed(DieselError::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                info,
            )) => Error::Database(DatabaseError::UniqueViolation(info.message().to_string())),
            StoragePgError::QueryFailed(DieselError::DatabaseError(
                diesel::result::DatabaseErrorKind::ForeignKeyViolation,
                info,
            )) => Error::Database(DatabaseError::ForeignKeyViolation(
                info.message().to_string(),
            )),
            StoragePgError::QueryFailed(e) => {
                Error::Database(DatabaseError::QueryFailed(e.to_string()))
            }
            StoragePgError::MigrationFailed(e) => {
                Error::Database(DatabaseError::MigrationFailed(e))
            }
            StoragePgError::SerializationError(e) => Error::Database(DatabaseError::Internal(e)),
            StoragePgError::CoreError(e) => Error::Database(DatabaseError::Internal(e)),
        }
    }
}

/// Extension trait to convert Diesel errors to core errors.
pub trait DieselErrorExt {
    /// Convert to a core Error type.
    fn into_core_error(self) -> Error;
}

impl DieselErrorExt for DieselError {
    fn into_core_error(self) -> Error {
        StoragePgError::QueryFailed(self).into()
    }
}

/// Helper function to convert a Diesel Result to a core Result.
pub fn map_diesel_err<T>(result: std::result::Result<T, DieselError>) -> whaleit_core::Result<T> {
    result.map_err(|e| e.into_core_error())
}

/// Extension trait for easily converting Results to core Results.
pub trait IntoCore<T> {
    fn into_core(self) -> whaleit_core::Result<T>;
}

impl<T> IntoCore<T> for std::result::Result<T, DieselError> {
    fn into_core(self) -> whaleit_core::Result<T> {
        self.map_err(|e| StoragePgError::from(e).into())
    }
}
