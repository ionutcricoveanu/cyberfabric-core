//! Domain error types

use data_ingestion_sdk::DataIngestionError;
use std::fmt;

#[derive(Debug)]
pub enum DomainError {
    Database(String),
    ExternalSource(String),
    NotFound(String),
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DomainError::Database(msg) => write!(f, "Database error: {}", msg),
            DomainError::ExternalSource(msg) => write!(f, "External source error: {}", msg),
            DomainError::NotFound(msg) => write!(f, "Not found: {}", msg),
        }
    }
}

impl std::error::Error for DomainError {}

impl From<DomainError> for DataIngestionError {
    fn from(err: DomainError) -> Self {
        match err {
            DomainError::Database(msg) => DataIngestionError::Database(msg),
            DomainError::ExternalSource(msg) => DataIngestionError::Source(msg),
            DomainError::NotFound(msg) => DataIngestionError::NotFound(msg),
        }
    }
}
