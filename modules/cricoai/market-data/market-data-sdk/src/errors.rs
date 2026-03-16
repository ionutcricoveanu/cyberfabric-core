use thiserror::Error;

/// Errors that can occur when interacting with the market-data service
#[derive(Debug, Error)]
pub enum MarketDataError {
    /// Symbol not found or invalid
    #[error("Invalid symbol: {0}")]
    InvalidSymbol(String),

    /// Invalid kline interval
    #[error("Invalid kline interval: {0}")]
    InvalidInterval(String),

    /// Database connection or query error
    #[error("Database error: {0}")]
    DatabaseError(String),

    /// External API error (Binance)
    #[error("Binance API error: {0}")]
    BinanceApiError(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded, retry after {0}s")]
    RateLimitExceeded(u64),

    /// Network connectivity issue
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Generic internal error
    #[error("Internal error: {0}")]
    InternalError(String),
}
