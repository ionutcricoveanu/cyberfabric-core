//! Error types for the trading-core module.

#[derive(Debug, thiserror::Error)]
pub enum TradingCoreError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("database error: {0}")]
    Database(String),

    #[error("exchange error: {0}")]
    Exchange(String),

    #[error("invalid state: {0}")]
    InvalidState(String),
}
