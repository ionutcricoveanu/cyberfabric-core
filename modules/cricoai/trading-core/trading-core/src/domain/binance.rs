//! Binance API client wrapper for trading operations.
//!
//! Handles both public endpoints (no auth required) and signed/authenticated
//! endpoints (HMAC-SHA256 signature + timestamp required by Binance).

use tracing::{info, warn, debug, error};
use reqwest::Client;
use rust_decimal::Decimal;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::domain::error::DomainError;
use crate::domain::models::{Quote, AccountBalance};
use crate::config::TradingCoreConfig;

type HmacSha256 = Hmac<Sha256>;

/// Order status information returned by the exchange
#[derive(Debug, Clone)]
pub struct OrderStatus {
    /// Exchange-assigned order ID
    pub order_id: String,
    /// NEW, PARTIALLY_FILLED, FILLED, CANCELED, EXPIRED
    pub status: String,
    pub filled_qty: f64,
    pub remaining_qty: f64,
    pub fill_price: Decimal,
}

/// Response from a newly placed order
#[derive(Debug, Clone)]
pub struct PlacedOrder {
    pub order_id: String,
    pub client_order_id: String,
    pub symbol: String,
    pub side: String,
    pub status: String,
    pub price: Decimal,
    pub orig_qty: f64,
    pub executed_qty: f64,
    pub transact_time_ms: i64,
}

/// Binance API client for trading operations.
/// Supports both testnet (`testnet.binance.vision`) and production (`api.binance.com`).
#[derive(Clone)]
pub struct BinanceApiClient {
    config: Arc<TradingCoreConfig>,
    http_client: Client,
    base_url: String,
}

impl BinanceApiClient {
    /// Create a new Binance API client
    pub fn new(config: Arc<TradingCoreConfig>) -> Self {
        let base_url = if config.testnet {
            "https://testnet.binance.vision".to_string()
        } else {
            "https://api.binance.com".to_string()
        };

        Self {
            config,
            http_client: Client::new(),
            base_url,
        }
    }

    // -------------------------------------------------------------------------
    // Helpers
    // -------------------------------------------------------------------------

    /// Current timestamp in milliseconds (required for signed requests)
    fn timestamp_ms() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is before UNIX epoch")
            .as_millis() as u64
    }

    /// Compute HMAC-SHA256 signature over the query string
    fn sign(&self, query: &str) -> Result<String, DomainError> {
        let secret = self.config.api_secret.as_bytes();
        let mut mac = HmacSha256::new_from_slice(secret)
            .map_err(|e| DomainError::BinanceApi(format!("HMAC key error: {e}")))?;
        mac.update(query.as_bytes());
        Ok(hex::encode(mac.finalize().into_bytes()))
    }

    /// Check that API credentials are configured
    fn require_credentials(&self) -> Result<(), DomainError> {
        if self.config.api_key.is_empty() || self.config.api_secret.is_empty() {
            return Err(DomainError::BinanceApi(
                "Binance API key/secret not configured".to_string(),
            ));
        }
        Ok(())
    }

    /// Parse a Binance response; surface the `msg` field on HTTP errors
    async fn parse_response(response: reqwest::Response) -> Result<serde_json::Value, DomainError> {
        let status = response.status();
        let text = response
            .text()
            .await
            .map_err(|e| DomainError::BinanceApi(format!("Failed to read response body: {e}")))?;

        let json: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| DomainError::BinanceApi(format!("Failed to parse JSON: {e} — body: {text}")))?;

        if !status.is_success() {
            let msg = json["msg"].as_str().unwrap_or("unknown error");
            let code = json["code"].as_i64().unwrap_or(0);
            return Err(DomainError::BinanceApi(format!(
                "Binance API error {code}: {msg}"
            )));
        }

        Ok(json)
    }

    // -------------------------------------------------------------------------
    // Public (unauthenticated) endpoints
    // -------------------------------------------------------------------------

    /// Check connectivity with Binance API (GET /api/v3/ping)
    pub async fn ping(&self) -> Result<bool, DomainError> {
        let url = format!("{}/api/v3/ping", self.base_url);

        match self.http_client.get(&url).send().await {
            Ok(response) => {
                let healthy = response.status().is_success();
                if !healthy {
                    warn!("Binance ping failed: status={}", response.status());
                }
                Ok(healthy)
            }
            Err(e) => {
                error!("Binance ping error: {}", e);
                Err(DomainError::BinanceApi(format!("API unreachable: {e}")))
            }
        }
    }

    /// Get current best bid/ask quote for a symbol (GET /api/v3/ticker/bookTicker)
    pub async fn get_quote(&self, symbol: &str) -> Result<Quote, DomainError> {
        let url = format!("{}/api/v3/ticker/bookTicker", self.base_url);

        let response = self
            .http_client
            .get(&url)
            .query(&[("symbol", symbol)])
            .send()
            .await
            .map_err(|e| DomainError::BinanceApi(format!("get_quote request failed: {e}")))?;

        let data = Self::parse_response(response).await?;

        let bid = Decimal::from_str_exact(data["bidPrice"].as_str().unwrap_or("0"))
            .unwrap_or(Decimal::ZERO);
        let ask = Decimal::from_str_exact(data["askPrice"].as_str().unwrap_or("0"))
            .unwrap_or(Decimal::ZERO);

        debug!("Quote for {}: bid={}, ask={}", symbol, bid, ask);

        Ok(Quote {
            symbol: symbol.to_string(),
            bid,
            ask,
            last_trade_price: bid,
            timestamp: chrono::Utc::now().naive_utc(),
        })
    }

    // -------------------------------------------------------------------------
    // Authenticated (signed) endpoints
    // -------------------------------------------------------------------------

    /// Get account balances (GET /api/v3/account — SIGNED)
    pub async fn get_account_balance(&self) -> Result<Vec<AccountBalance>, DomainError> {
        self.require_credentials()?;

        let ts = Self::timestamp_ms();
        let query = format!("timestamp={ts}&recvWindow=5000");
        let sig = self.sign(&query)?;
        let signed_query = format!("{query}&signature={sig}");

        let url = format!("{}/api/v3/account?{signed_query}", self.base_url);

        debug!("Fetching account balances");

        let response = self
            .http_client
            .get(&url)
            .header("X-MBX-APIKEY", &self.config.api_key)
            .send()
            .await
            .map_err(|e| DomainError::BinanceApi(format!("get_account_balance request failed: {e}")))?;

        let data = Self::parse_response(response).await?;

        let balances = data["balances"]
            .as_array()
            .ok_or_else(|| DomainError::BinanceApi("Missing 'balances' in account response".to_string()))?
            .iter()
            .filter_map(|b| {
                let asset = b["asset"].as_str()?.to_string();
                let free = Decimal::from_str_exact(b["free"].as_str().unwrap_or("0")).ok()?;
                let locked = Decimal::from_str_exact(b["locked"].as_str().unwrap_or("0")).ok()?;
                // Skip zero balances to reduce noise
                if free == Decimal::ZERO && locked == Decimal::ZERO {
                    return None;
                }
                Some(AccountBalance {
                    asset,
                    free,
                    locked,
                    total: free + locked,
                })
            })
            .collect();

        Ok(balances)
    }

    /// Place a limit order (POST /api/v3/order — SIGNED)
    ///
    /// Returns the exchange order ID on success.
    pub async fn place_limit_order(
        &self,
        symbol: &str,
        side: &str,      // "BUY" | "SELL"
        quantity: f64,
        price: Decimal,
    ) -> Result<PlacedOrder, DomainError> {
        if !self.config.trading_enabled {
            return Err(DomainError::TradingDisabled);
        }
        self.require_credentials()?;

        let ts = Self::timestamp_ms();
        // Format quantity/price with enough precision; Binance rejects trailing zeros in some cases
        let qty_str = format!("{:.8}", quantity).trim_end_matches('0').trim_end_matches('.').to_string();
        let price_str = price.to_string();

        let query = format!(
            "symbol={symbol}&side={side}&type=LIMIT&timeInForce=GTC\
             &quantity={qty_str}&price={price_str}\
             &timestamp={ts}&recvWindow=5000"
        );
        let sig = self.sign(&query)?;
        let body = format!("{query}&signature={sig}");

        info!("Placing LIMIT {} order: symbol={} qty={} price={}", side, symbol, qty_str, price_str);

        let url = format!("{}/api/v3/order", self.base_url);
        let response = self
            .http_client
            .post(&url)
            .header("X-MBX-APIKEY", &self.config.api_key)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .map_err(|e| DomainError::BinanceApi(format!("place_limit_order request failed: {e}")))?;

        let data = Self::parse_response(response).await?;

        let order_id = data["orderId"]
            .as_i64()
            .map(|id| id.to_string())
            .or_else(|| data["orderId"].as_str().map(str::to_string))
            .unwrap_or_default();

        let placed = PlacedOrder {
            order_id: order_id.clone(),
            client_order_id: data["clientOrderId"].as_str().unwrap_or("").to_string(),
            symbol: data["symbol"].as_str().unwrap_or(symbol).to_string(),
            side: data["side"].as_str().unwrap_or(side).to_string(),
            status: data["status"].as_str().unwrap_or("NEW").to_string(),
            price: Decimal::from_str_exact(data["price"].as_str().unwrap_or(&price_str))
                .unwrap_or(price),
            orig_qty: data["origQty"]
                .as_str()
                .and_then(|s| s.parse().ok())
                .unwrap_or(quantity),
            executed_qty: data["executedQty"]
                .as_str()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0.0),
            transact_time_ms: data["transactTime"].as_i64().unwrap_or(ts as i64),
        };

        info!("Order placed: id={} status={}", placed.order_id, placed.status);
        Ok(placed)
    }

    /// Cancel an open order (DELETE /api/v3/order — SIGNED)
    pub async fn cancel_order(&self, symbol: &str, order_id: &str) -> Result<(), DomainError> {
        self.require_credentials()?;

        let ts = Self::timestamp_ms();
        let query = format!("symbol={symbol}&orderId={order_id}&timestamp={ts}&recvWindow=5000");
        let sig = self.sign(&query)?;
        let body = format!("{query}&signature={sig}");

        info!("Cancelling order {} for {}", order_id, symbol);

        let url = format!("{}/api/v3/order", self.base_url);
        let response = self
            .http_client
            .delete(&url)
            .header("X-MBX-APIKEY", &self.config.api_key)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .map_err(|e| DomainError::BinanceApi(format!("cancel_order request failed: {e}")))?;

        Self::parse_response(response).await?;
        Ok(())
    }

    /// Query order status (GET /api/v3/order — SIGNED)
    pub async fn get_order_status(
        &self,
        symbol: &str,
        order_id: &str,
    ) -> Result<OrderStatus, DomainError> {
        self.require_credentials()?;

        let ts = Self::timestamp_ms();
        let query = format!("symbol={symbol}&orderId={order_id}&timestamp={ts}&recvWindow=5000");
        let sig = self.sign(&query)?;
        let signed_query = format!("{query}&signature={sig}");

        debug!("Checking order {} status for {}", order_id, symbol);

        let url = format!("{}/api/v3/order?{signed_query}", self.base_url);
        let response = self
            .http_client
            .get(&url)
            .header("X-MBX-APIKEY", &self.config.api_key)
            .send()
            .await
            .map_err(|e| DomainError::BinanceApi(format!("get_order_status request failed: {e}")))?;

        let data = Self::parse_response(response).await?;

        let order_id_str = data["orderId"]
            .as_i64()
            .map(|id| id.to_string())
            .or_else(|| data["orderId"].as_str().map(str::to_string))
            .unwrap_or_else(|| order_id.to_string());

        Ok(OrderStatus {
            order_id: order_id_str,
            status: data["status"].as_str().unwrap_or("UNKNOWN").to_string(),
            filled_qty: data["executedQty"]
                .as_str()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0.0),
            remaining_qty: {
                let orig: f64 = data["origQty"].as_str().and_then(|s| s.parse().ok()).unwrap_or(0.0);
                let exec: f64 = data["executedQty"].as_str().and_then(|s| s.parse().ok()).unwrap_or(0.0);
                orig - exec
            },
            fill_price: Decimal::from_str_exact(
                data["cummulativeQuoteQty"].as_str().unwrap_or("0"),
            )
            .unwrap_or(Decimal::ZERO),
        })
    }
}
