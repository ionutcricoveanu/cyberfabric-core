//! Error types for the market-data module.

#[derive(Debug, thiserror::Error)]
pub enum MarketDataError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("database error: {0}")]
    Database(String),

    #[error("exchange error: {0}")]
    Exchange(String),
}
