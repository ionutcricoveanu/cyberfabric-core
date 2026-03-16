//! Market Data SDK
//!
//! This crate provides the public API for the `market-data` module:
//! - `MarketDataApi` trait
//! - Model types for klines (candlesticks), tickers, order books
//! - Error type (`MarketDataError`)
//!
//! ## Usage
//!
//! Consumers obtain the client from `ClientHub`:
//! ```ignore
//! use market_data_sdk::MarketDataApi;
//!
//! let client = hub.get::<dyn MarketDataApi>()?;
//! let klines = client.get_klines(&ctx, "BTCUSDT", "1h", Some(100)).await?;
//! ```

#![forbid(unsafe_code)]
#![deny(rust_2018_idioms)]

pub mod client;
pub mod errors;
pub mod models;

pub use client::MarketDataApi;
pub use errors::MarketDataError;
pub use models::{Kline, KlineInterval, Ticker24h};
