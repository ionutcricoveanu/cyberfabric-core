//! Error types for the config-manager module.

#[derive(Debug, thiserror::Error)]
pub enum ConfigManagerError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("database error: {0}")]
    Database(String),

    #[error("invalid configuration: {0}")]
    InvalidConfig(String),
}
