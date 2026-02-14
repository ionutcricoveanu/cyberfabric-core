//! Error types for the data-ingestion module.

#[derive(Debug, thiserror::Error)]
pub enum DataIngestionError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("database error: {0}")]
    Database(String),

    #[error("source error: {0}")]
    Source(String),
}
