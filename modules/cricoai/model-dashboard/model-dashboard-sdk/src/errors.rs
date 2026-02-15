use thiserror::Error;

#[derive(Debug, Error)]
pub enum ModelDashboardError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}
