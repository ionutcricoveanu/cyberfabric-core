//! Trading Dashboard SDK
//!
//! This crate provides the public API for the `trading-dashboard` module:
//! - `TradingDashboardApi` trait
//! - Model types for trading stats, orders, P&L
//! - Error type (`TradingDashboardError`)
//! - OData filter field definitions (behind `odata` feature)
//!
//! ## Usage
//!
//! Consumers obtain the client from `ClientHub`:
//! ```ignore
//! use trading_dashboard_sdk::TradingDashboardApi;
//!
//! let client = hub.get::<dyn TradingDashboardApi>()?;
//! let summary = client.get_stats_summary(&ctx, "24h").await?;
//! ```

#![forbid(unsafe_code)]
#![deny(rust_2018_idioms)]

pub mod client;
pub mod errors;
pub mod models;

#[cfg(feature = "odata")]
pub mod odata;

pub use client::TradingDashboardApi;
pub use errors::TradingDashboardError;
pub use models::{DailyPnl, StatsSummary, TradeRecord};
