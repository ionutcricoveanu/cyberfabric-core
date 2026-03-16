//! Market Data Module
//!
//! Collects and stores kline (candlestick) data from Binance.
//! Runs as a background service with lifecycle management.

#![forbid(unsafe_code)]
#![deny(rust_2018_idioms)]

pub mod api;
pub mod config;
pub mod domain;
pub mod module;

// Re-export for convenience
pub use module::MarketData;
