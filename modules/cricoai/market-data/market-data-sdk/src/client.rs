//! Object-safe client trait for the market-data module.

use async_trait::async_trait;
use modkit_security::SecurityContext;

use crate::errors::MarketDataError;
use crate::models::{Kline, Price};

#[async_trait]
pub trait MarketDataApi: Send + Sync {
    /// Get the latest price for a symbol.
    async fn get_latest_price(
        &self,
        ctx: &SecurityContext,
        symbol: &str,
    ) -> Result<Price, MarketDataError>;

    /// Get kline (candlestick) data for a symbol and interval.
    async fn get_klines(
        &self,
        ctx: &SecurityContext,
        symbol: &str,
        interval: &str,
        limit: u32,
    ) -> Result<Vec<Kline>, MarketDataError>;
}
