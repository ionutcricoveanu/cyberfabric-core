use config_manager_sdk::errors::ConfigManagerError;

#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("database error: {0}")]
    Database(String),

    #[error("invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("io error: {0}")]
    Io(String),
}

impl From<DomainError> for ConfigManagerError {
    fn from(e: DomainError) -> Self {
        match e {
            DomainError::NotFound(msg) => Self::NotFound(msg),
            DomainError::Database(msg) => Self::Database(msg),
            DomainError::InvalidConfig(msg) => Self::InvalidConfig(msg),
            DomainError::Io(msg) => Self::Database(msg),
        }
    }
}
