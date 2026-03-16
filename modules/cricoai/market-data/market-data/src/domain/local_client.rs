use std::sync::Arc;

use async_trait::async_trait;
use modkit_security::SecurityContext;

use market_data_sdk::{MarketDataApi, MarketDataError, Kline, KlineInterval, Ticker24h};
use crate::domain::service::MarketDataService;

/// Local client implementation of MarketDataApi for ClientHub registration
///
/// This wraps the MarketDataService and exposes it through the MarketDataApi trait,
/// allowing other modules to access market data functionality via ClientHub.
pub struct MarketDataLocalClient {
    service: Arc<MarketDataService>,
}

impl MarketDataLocalClient {
    pub fn new(service: Arc<MarketDataService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl MarketDataApi for MarketDataLocalClient {
    async fn get_klines(
        &self,
        ctx: &SecurityContext,
        symbol: &str,
        interval: KlineInterval,
        limit: Option<u32>,
    ) -> Result<Vec<Kline>, MarketDataError> {
        self.service.get_klines(ctx, symbol, interval, limit).await
    }

    async fn get_latest_kline(
        &self,
        ctx: &SecurityContext,
        symbol: &str,
        interval: KlineInterval,
    ) -> Result<Option<Kline>, MarketDataError> {
        self.service.get_latest_kline(ctx, symbol, interval).await
    }

    async fn get_ticker_24h(
        &self,
        ctx: &SecurityContext,
        symbol: &str,
    ) -> Result<Ticker24h, MarketDataError> {
        self.service.get_ticker_24h(ctx, symbol).await
    }

    async fn get_all_tickers_24h(
        &self,
        ctx: &SecurityContext,
    ) -> Result<Vec<Ticker24h>, MarketDataError> {
        self.service.get_all_tickers_24h(ctx).await
    }

    async fn has_klines(
        &self,
        ctx: &SecurityContext,
        symbol: &str,
        interval: KlineInterval,
    ) -> Result<bool, MarketDataError> {
        self.service.has_klines(ctx, symbol, interval).await
    }
}
