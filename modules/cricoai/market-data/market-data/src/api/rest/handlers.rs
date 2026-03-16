use axum::extract::{Path, Query};
use axum::Extension;
use modkit::api::prelude::*;
use modkit_security::SecurityContext;
use serde::Deserialize;
use std::sync::Arc;
use tracing::info;

use crate::api::rest::error::to_problem;
use crate::domain::service::MarketDataService;
use market_data_sdk::models::KlineInterval;

/// Query parameters for klines endpoint
#[derive(Debug, Deserialize)]
pub struct KlinesQuery {
    /// Kline interval (1m, 5m, 15m, 1h, 4h)
    pub interval: String,

    /// Maximum number of klines to return (default: 100, max: 1000)
    #[serde(default = "default_limit")]
    pub limit: u32,
}

fn default_limit() -> u32 {
    100
}

/// Response DTO for klines
#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct KlinesResponse {
    pub symbol: String,
    pub interval: String,
    pub klines: Vec<market_data_sdk::models::Kline>,
}

/// Response DTO for single ticker
#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct Ticker24hResponse {
    pub symbol: String,
    pub price_change: f64,
    pub price_change_percent: f64,
    pub weighted_avg_price: f64,
    pub last_price: f64,
    pub last_qty: f64,
    pub bid_price: f64,
    pub ask_price: f64,
    pub open_price: f64,
    pub high_price: f64,
    pub low_price: f64,
    pub volume: f64,
    pub quote_volume: f64,
    pub open_time: chrono::DateTime<chrono::Utc>,
    pub close_time: chrono::DateTime<chrono::Utc>,
    pub count: i64,
}

/// Response DTO for multiple tickers
#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct AllTickers24hResponse {
    pub tickers: Vec<Ticker24hResponse>,
}

impl From<market_data_sdk::models::Ticker24h> for Ticker24hResponse {
    fn from(ticker: market_data_sdk::models::Ticker24h) -> Self {
        Self {
            symbol: ticker.symbol,
            price_change: ticker.price_change,
            price_change_percent: ticker.price_change_percent,
            weighted_avg_price: ticker.weighted_avg_price,
            last_price: ticker.last_price,
            last_qty: ticker.last_qty,
            bid_price: ticker.bid_price,
            ask_price: ticker.ask_price,
            open_price: ticker.open_price,
            high_price: ticker.high_price,
            low_price: ticker.low_price,
            volume: ticker.volume,
            quote_volume: ticker.quote_volume,
            open_time: ticker.open_time,
            close_time: ticker.close_time,
            count: ticker.count,
        }
    }
}

/// GET /market-data/v1/klines/{symbol}
///
/// Fetch historical klines for a specific symbol and interval
pub(crate) async fn get_klines(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<MarketDataService>>,
    Path(symbol): Path<String>,
    Query(query): Query<KlinesQuery>,
) -> ApiResult<JsonBody<KlinesResponse>> {
    info!(
        subject_id = %ctx.subject_id(),
        symbol = %symbol,
        interval = %query.interval,
        "GET /market-data/v1/klines/{symbol}"
    );

    // Parse and validate interval
    let interval = KlineInterval::from_str(&query.interval)
        .ok_or_else(|| {
            modkit_errors::Problem::new(
                http::StatusCode::BAD_REQUEST,
                "Invalid Interval",
                format!("Invalid interval: {}", query.interval)
            )
        })?;

    // Validate limit
    let limit = query.limit.min(1000);

    // Fetch klines from service
    let klines = svc
        .get_klines(&ctx, &symbol, interval, Some(limit))
        .await
        .map_err(to_problem)?;

    Ok(Json(KlinesResponse {
        symbol,
        interval: query.interval,
        klines,
    }))
}

/// GET /market-data/v1/ticker/{symbol}
///
/// Fetch 24-hour ticker statistics for a specific symbol
pub(crate) async fn get_ticker_24h(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<MarketDataService>>,
    Path(symbol): Path<String>,
) -> ApiResult<JsonBody<Ticker24hResponse>> {
    info!(
        subject_id = %ctx.subject_id(),
        symbol = %symbol,
        "GET /market-data/v1/ticker/{symbol}"
    );

    let ticker = svc
        .get_ticker_24h(&ctx, &symbol)
        .await
        .map_err(to_problem)?;

    Ok(Json(ticker.into()))
}

/// GET /market-data/v1/tickers
///
/// Fetch 24-hour ticker statistics for all symbols
pub(crate) async fn get_all_tickers_24h(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<MarketDataService>>,
) -> ApiResult<JsonBody<AllTickers24hResponse>> {
    info!(
        subject_id = %ctx.subject_id(),
        "GET /market-data/v1/tickers"
    );

    let tickers = svc
        .get_all_tickers_24h(&ctx)
        .await
        .map_err(to_problem)?;

    let response = AllTickers24hResponse {
        tickers: tickers.into_iter().map(Into::into).collect(),
    };

    Ok(Json(response))
}
