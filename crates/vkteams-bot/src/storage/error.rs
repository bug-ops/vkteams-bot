//! Storage error types

use thiserror::Error;

/// Storage operation errors
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("Database connection error: {0}")]
    Connection(String),

    #[error("Database query error: {0}")]
    Query(String),

    #[error("Vector store error: {0}")]
    Vector(String),

    #[error("Embedding generation error: {0}")]
    Embedding(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Migration error: {0}")]
    Migration(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Unexpected error: {0}")]
    Unexpected(String),
}

#[cfg(feature = "database")]
impl From<sqlx::Error> for StorageError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => StorageError::NotFound("Record not found".to_string()),
            sqlx::Error::Database(db_err) => StorageError::Query(db_err.message().to_string()),
            sqlx::Error::PoolClosed | sqlx::Error::PoolTimedOut => {
                StorageError::Connection(err.to_string())
            }
            _ => StorageError::Query(err.to_string()),
        }
    }
}

/// Result type for storage operations
pub type StorageResult<T> = Result<T, StorageError>;
