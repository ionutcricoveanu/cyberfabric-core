//! Market Data SDK
//!
//! Public API for the `market-data` module:
//! - `MarketDataApi` trait
//! - Model types for prices and klines
//! - Error type (`MarketDataError`)

#![forbid(unsafe_code)]
#![deny(rust_2018_idioms)]

pub mod client;
pub mod errors;
pub mod models;

pub use client::MarketDataApi;
pub use errors::MarketDataError;
pub use models::{Kline, Price};
