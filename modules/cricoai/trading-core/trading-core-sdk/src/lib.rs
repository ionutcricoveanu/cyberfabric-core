//! Trading Core SDK
//!
//! Public API for the `trading-core` module:
//! - `TradingCoreApi` trait
//! - Model types for bot status
//! - Error type (`TradingCoreError`)

#![forbid(unsafe_code)]
#![deny(rust_2018_idioms)]

pub mod client;
pub mod errors;
pub mod models;

pub use client::TradingCoreApi;
pub use errors::TradingCoreError;
pub use models::BotStatus;
