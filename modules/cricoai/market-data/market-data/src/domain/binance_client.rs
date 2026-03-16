use chrono::DateTime;
use market_data_sdk::models::{Kline, Ticker24h};
use market_data_sdk::MarketDataError;
use reqwest::Client;
use serde::Deserialize;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

/// Binance REST API client for market data
#[derive(Clone)]
pub struct BinanceClient {
    client: Client,
    base_url: String,
    max_retries: u32,
}

/// Raw kline response from Binance API
#[derive(Debug, Deserialize)]
struct BinanceKline {
    #[serde(rename = "0")]
    open_time: i64,
    #[serde(rename = "1")]
    open: String,
    #[serde(rename = "2")]
    high: String,
    #[serde(rename = "3")]
    low: String,
    #[serde(rename = "4")]
    close: String,
    #[serde(rename = "5")]
    volume: String,
    #[serde(rename = "6")]
    close_time: i64,
    #[serde(rename = "7")]
    quote_volume: String,
    #[serde(rename = "8")]
    trades: i32,
    #[serde(rename = "9")]
    taker_buy_base: String,
    #[serde(rename = "10")]
    taker_buy_quote: String,
}

/// Raw ticker response from Binance API
#[derive(Debug, Deserialize)]
struct BinanceTicker {
    symbol: String,
    #[serde(rename = "priceChange")]
    price_change: String,
    #[serde(rename = "priceChangePercent")]
    price_change_percent: String,
    #[serde(rename = "weightedAvgPrice")]
    weighted_avg_price: String,
    #[serde(rename = "lastPrice")]
    last_price: String,
    #[serde(rename = "lastQty")]
    last_qty: String,
    #[serde(rename = "bidPrice")]
    bid_price: String,
    #[serde(rename = "askPrice")]
    ask_price: String,
    #[serde(rename = "openPrice")]
    open_price: String,
    #[serde(rename = "highPrice")]
    high_price: String,
    #[serde(rename = "lowPrice")]
    low_price: String,
    volume: String,
    #[serde(rename = "quoteVolume")]
    quote_volume: String,
    #[serde(rename = "openTime")]
    open_time: i64,
    #[serde(rename = "closeTime")]
    close_time: i64,
    count: i64,
}

impl BinanceClient {
    /// Create a new Binance client
    pub fn new(base_url: String) -> Result<Arc<Self>, MarketDataError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| MarketDataError::InternalError(format!("Failed to create HTTP client: {e}")))?;

        Ok(Arc::new(Self {
            client,
            base_url,
            max_retries: 3,
        }))
    }

    /// Fetch klines from Binance with retry logic
    pub async fn fetch_klines(
        &self,
        symbol: &str,
        interval: &str,
        limit: u32,
    ) -> Result<Vec<Kline>, MarketDataError> {
        let url = format!("{}/api/v3/klines", self.base_url);
        
        for attempt in 1..=self.max_retries {
            match self.try_fetch_klines(&url, symbol, interval, limit).await {
                Ok(klines) => {
                    debug!(
                        "Fetched {} klines for {} ({})",
                        klines.len(),
                        symbol,
                        interval
                    );
                    return Ok(klines);
                }
                Err(e) => {
                    if attempt < self.max_retries {
                        let backoff = Duration::from_secs(2u64.pow(attempt - 1));
                        warn!(
                            "Failed to fetch klines for {} (attempt {}/{}): {}. Retrying in {:?}",
                            symbol, attempt, self.max_retries, e, backoff
                        );
                        sleep(backoff).await;
                    } else {
                        error!("Failed to fetch klines for {} after {} attempts: {}", symbol, self.max_retries, e);
                        return Err(e);
                    }
                }
            }
        }

        Err(MarketDataError::InternalError("Unexpected retry loop exit".to_string()))
    }

    async fn try_fetch_klines(
        &self,
        url: &str,
        symbol: &str,
        interval: &str,
        limit: u32,
    ) -> Result<Vec<Kline>, MarketDataError> {
        let response = self
            .client
            .get(url)
            .query(&[
                ("symbol", symbol),
                ("interval", interval),
                ("limit", &limit.to_string()),
            ])
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    MarketDataError::NetworkError(format!("Request timeout: {e}"))
                } else if e.is_connect() {
                    MarketDataError::NetworkError(format!("Connection error: {e}"))
                } else {
                    MarketDataError::BinanceApiError(format!("Request failed: {e}"))
                }
            })?;

        // Check for rate limiting
        if response.status() == 429 {
            let retry_after = response
                .headers()
                .get("Retry-After")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(60);
            return Err(MarketDataError::RateLimitExceeded(retry_after));
        }

        // Check for other errors
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(MarketDataError::BinanceApiError(format!(
                "HTTP {}: {}",
                status, body
            )));
        }

        let raw_klines: Vec<BinanceKline> = response
            .json()
            .await
            .map_err(|e| MarketDataError::BinanceApiError(format!("Failed to parse response: {e}")))?;

        let klines = raw_klines
            .into_iter()
            .map(|k| self.convert_kline(k, symbol, interval))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(klines)
    }

    fn convert_kline(
        &self,
        raw: BinanceKline,
        symbol: &str,
        interval: &str,
    ) -> Result<Kline, MarketDataError> {
        Ok(Kline {
            start_time: DateTime::from_timestamp_millis(raw.open_time)
                .ok_or_else(|| MarketDataError::InternalError("Invalid start_time".to_string()))?,
            close_time: DateTime::from_timestamp_millis(raw.close_time)
                .ok_or_else(|| MarketDataError::InternalError("Invalid close_time".to_string()))?,
            symbol: symbol.to_string(),
            interval: interval.to_string(),
            open_price: raw.open.parse().map_err(|e| {
                MarketDataError::InternalError(format!("Failed to parse open_price: {e}"))
            })?,
            close_price: raw.close.parse().map_err(|e| {
                MarketDataError::InternalError(format!("Failed to parse close_price: {e}"))
            })?,
            high_price: raw.high.parse().map_err(|e| {
                MarketDataError::InternalError(format!("Failed to parse high_price: {e}"))
            })?,
            low_price: raw.low.parse().map_err(|e| {
                MarketDataError::InternalError(format!("Failed to parse low_price: {e}"))
            })?,
            base_asset_volume: raw.volume.parse().map_err(|e| {
                MarketDataError::InternalError(format!("Failed to parse volume: {e}"))
            })?,
            quote_asset_volume: raw.quote_volume.parse().map_err(|e| {
                MarketDataError::InternalError(format!("Failed to parse quote_volume: {e}"))
            })?,
            number_of_trades: raw.trades,
            taker_buy_base_asset_volume: raw.taker_buy_base.parse().map_err(|e| {
                MarketDataError::InternalError(format!("Failed to parse taker_buy_base: {e}"))
            })?,
            taker_buy_quote_asset_volume: raw.taker_buy_quote.parse().map_err(|e| {
                MarketDataError::InternalError(format!("Failed to parse taker_buy_quote: {e}"))
            })?,
            kline_closed: true, // Only closed klines from historical endpoint
        })
    }

    /// Fetch 24h ticker for a symbol
    pub async fn fetch_ticker_24h(&self, symbol: &str) -> Result<Ticker24h, MarketDataError> {
        let url = format!("{}/api/v3/ticker/24hr", self.base_url);

        let response = self
            .client
            .get(&url)
            .query(&[("symbol", symbol)])
            .send()
            .await
            .map_err(|e| MarketDataError::NetworkError(format!("Request failed: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(MarketDataError::BinanceApiError(format!(
                "HTTP {}: {}",
                status, body
            )));
        }

        let raw: BinanceTicker = response
            .json()
            .await
            .map_err(|e| MarketDataError::BinanceApiError(format!("Failed to parse response: {e}")))?;

        self.convert_ticker(raw)
    }

    /// Fetch 24h tickers for all symbols
    pub async fn fetch_all_tickers_24h(&self) -> Result<Vec<Ticker24h>, MarketDataError> {
        let url = format!("{}/api/v3/ticker/24hr", self.base_url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| MarketDataError::NetworkError(format!("Request failed: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(MarketDataError::BinanceApiError(format!(
                "HTTP {}: {}",
                status, body
            )));
        }

        let raw_tickers: Vec<BinanceTicker> = response
            .json()
            .await
            .map_err(|e| MarketDataError::BinanceApiError(format!("Failed to parse response: {e}")))?;

        raw_tickers
            .into_iter()
            .map(|t| self.convert_ticker(t))
            .collect()
    }

    fn convert_ticker(&self, raw: BinanceTicker) -> Result<Ticker24h, MarketDataError> {
        Ok(Ticker24h {
            symbol: raw.symbol,
            price_change: raw.price_change.parse().map_err(|e| {
                MarketDataError::InternalError(format!("Failed to parse price_change: {e}"))
            })?,
            price_change_percent: raw.price_change_percent.parse().map_err(|e| {
                MarketDataError::InternalError(format!("Failed to parse price_change_percent: {e}"))
            })?,
            weighted_avg_price: raw.weighted_avg_price.parse().map_err(|e| {
                MarketDataError::InternalError(format!("Failed to parse weighted_avg_price: {e}"))
            })?,
            last_price: raw.last_price.parse().map_err(|e| {
                MarketDataError::InternalError(format!("Failed to parse last_price: {e}"))
            })?,
            last_qty: raw.last_qty.parse().map_err(|e| {
                MarketDataError::InternalError(format!("Failed to parse last_qty: {e}"))
            })?,
            bid_price: raw.bid_price.parse().map_err(|e| {
                MarketDataError::InternalError(format!("Failed to parse bid_price: {e}"))
            })?,
            ask_price: raw.ask_price.parse().map_err(|e| {
                MarketDataError::InternalError(format!("Failed to parse ask_price: {e}"))
            })?,
            open_price: raw.open_price.parse().map_err(|e| {
                MarketDataError::InternalError(format!("Failed to parse open_price: {e}"))
            })?,
            high_price: raw.high_price.parse().map_err(|e| {
                MarketDataError::InternalError(format!("Failed to parse high_price: {e}"))
            })?,
            low_price: raw.low_price.parse().map_err(|e| {
                MarketDataError::InternalError(format!("Failed to parse low_price: {e}"))
            })?,
            volume: raw.volume.parse().map_err(|e| {
                MarketDataError::InternalError(format!("Failed to parse volume: {e}"))
            })?,
            quote_volume: raw.quote_volume.parse().map_err(|e| {
                MarketDataError::InternalError(format!("Failed to parse quote_volume: {e}"))
            })?,
            open_time: DateTime::from_timestamp_millis(raw.open_time)
                .ok_or_else(|| MarketDataError::InternalError("Invalid open_time".to_string()))?,
            close_time: DateTime::from_timestamp_millis(raw.close_time)
                .ok_or_else(|| MarketDataError::InternalError("Invalid close_time".to_string()))?,
            count: raw.count,
        })
    }

    /// Test connectivity to Binance API
    pub async fn test_connectivity(&self) -> Result<(), MarketDataError> {
        let url = format!("{}/api/v3/ping", self.base_url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| MarketDataError::NetworkError(format!("Ping failed: {e}")))?;

        if response.status().is_success() {
            info!("Binance API connectivity test: OK");
            Ok(())
        } else {
            Err(MarketDataError::NetworkError(format!(
                "Ping returned {}",
                response.status()
            )))
        }
    }
}
