//! Error types for the auth-manager module.

#[derive(Debug, thiserror::Error)]
pub enum AuthManagerError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("database error: {0}")]
    Database(String),

    #[error("unauthorized: {0}")]
    Unauthorized(String),

    #[error("conflict: {0}")]
    Conflict(String),
}
