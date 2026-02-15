use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthManagementError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),
}
