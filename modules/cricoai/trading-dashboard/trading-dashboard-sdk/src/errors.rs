//! Error types for the trading-dashboard module.

/// Domain errors for trading dashboard operations.
#[derive(Debug, thiserror::Error)]
pub enum TradingDashboardError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("database error: {0}")]
    Database(String),

    #[error("invalid query: {0}")]
    InvalidQuery(String),
}
