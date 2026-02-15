use std::fmt;

#[derive(Debug)]
pub enum DomainError {
    Database(String),
    NotFound(String),
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(msg) => write!(f, "Database error: {msg}"),
            Self::NotFound(msg) => write!(f, "Not found: {msg}"),
        }
    }
}

impl std::error::Error for DomainError {}
