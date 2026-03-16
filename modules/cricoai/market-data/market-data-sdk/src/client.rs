use async_trait::async_trait;
use modkit_security::SecurityContext;

use crate::errors::MarketDataError;
use crate::models::{Kline, KlineInterval, Ticker24h};

/// Market Data API trait
///
/// Provides access to:
/// - Historical kline (candlestick) data
/// - Real-time ticker information
/// - Market statistics
#[async_trait]
pub trait MarketDataApi: Send + Sync + 'static {
    /// Get klines for a specific symbol and interval
    ///
    /// # Arguments
    /// * `ctx` - Security context for authentication and tenant isolation
    /// * `symbol` - Trading pair symbol (e.g., "BTCUSDT")
    /// * `interval` - Kline interval (e.g., "1m", "1h", "4h")
    /// * `limit` - Maximum number of klines to return (default: 100, max: 1000)
    ///
    /// # Returns
    /// Vector of klines ordered by start_time (oldest first)
    async fn get_klines(
        &self,
        ctx: &SecurityContext,
        symbol: &str,
        interval: KlineInterval,
        limit: Option<u32>,
    ) -> Result<Vec<Kline>, MarketDataError>;

    /// Get the latest kline for a symbol
    ///
    /// # Arguments
    /// * `ctx` - Security context for authentication and tenant isolation
    /// * `symbol` - Trading pair symbol
    /// * `interval` - Kline interval
    async fn get_latest_kline(
        &self,
        ctx: &SecurityContext,
        symbol: &str,
        interval: KlineInterval,
    ) -> Result<Option<Kline>, MarketDataError>;

    /// Get 24-hour ticker statistics for a symbol
    ///
    /// # Arguments
    /// * `ctx` - Security context for authentication and tenant isolation
    /// * `symbol` - Trading pair symbol
    async fn get_ticker_24h(
        &self,
        ctx: &SecurityContext,
        symbol: &str,
    ) -> Result<Ticker24h, MarketDataError>;

    /// Get 24-hour ticker statistics for all symbols
    ///
    /// # Arguments
    /// * `ctx` - Security context for authentication and tenant isolation
    async fn get_all_tickers_24h(
        &self,
        ctx: &SecurityContext,
    ) -> Result<Vec<Ticker24h>, MarketDataError>;

    /// Check if klines exist for a symbol and interval
    ///
    /// # Arguments
    /// * `ctx` - Security context for authentication and tenant isolation
    /// * `symbol` - Trading pair symbol
    /// * `interval` - Kline interval
    async fn has_klines(
        &self,
        ctx: &SecurityContext,
        symbol: &str,
        interval: KlineInterval,
    ) -> Result<bool, MarketDataError>;
}
