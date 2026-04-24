use thiserror::Error;

/// Custom error type for activity-related operations
#[derive(Debug, Error)]
pub enum ActivityError {
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Invalid data: {0}")]
    InvalidData(String),
    #[error("Asset error: {0}")]
    AssetError(String),
    #[error("Currency exchange error: {0}")]
    CurrencyExchangeError(String),
}

impl From<ActivityError> for String {
    fn from(error: ActivityError) -> Self {
        error.to_string()
    }
}
