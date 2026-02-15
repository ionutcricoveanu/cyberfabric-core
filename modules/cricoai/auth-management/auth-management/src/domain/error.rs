use std::fmt;

#[derive(Debug)]
pub enum DomainError {
    Database(String),
    NotFound(String),
    Forbidden(String),
    BadRequest(String),
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(msg) => write!(f, "Database error: {msg}"),
            Self::NotFound(msg) => write!(f, "Not found: {msg}"),
            Self::Forbidden(msg) => write!(f, "Forbidden: {msg}"),
            Self::BadRequest(msg) => write!(f, "Bad request: {msg}"),
        }
    }
}

impl std::error::Error for DomainError {}
