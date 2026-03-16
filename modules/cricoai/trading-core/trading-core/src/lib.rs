//! Trading Core Module Implementation
//!
//! Main trading engine with buy/sell logic, Binance integration, and risk management.
//! Implements staggered buy/sell strategy execution with integration to:
//! - market-data (for kline and ticker data)
//! - ml-service (for predictions)
//! - market-intelligence (for agent decisions)
//! - config-manager (for trading configuration)

#![forbid(unsafe_code)]
#![deny(rust_2018_idioms)]

pub mod module;
pub mod config;
pub mod api;
pub mod domain;
#[cfg(test)]
pub mod tests;

pub use module::TradingCoreModule;
pub use config::TradingCoreConfig;
