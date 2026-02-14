//! Trading Dashboard Module
//!
//! Read-only REST API serving the HAI3 frontend with trading data,
//! model metrics, agent analytics, and performance monitoring.
//!
//! ## Architecture
//!
//! ### Contract Layer (`trading-dashboard-sdk`)
//! - `TradingDashboardApi` trait
//! - Model types: `StatsSummary`, `DailyPnl`, `TradeRecord`
//! - Error type: `TradingDashboardError`
//!
//! ### API Layer (`api/`)
//! - `routes/` — Per-resource route definitions
//! - `handlers/` — Request handlers
//! - `dto.rs` — REST DTOs with serde + utoipa
//! - `error.rs` — Domain errors → RFC 9457 Problem
//!
//! ### Domain Layer (`domain/`)
//! - `service/` — Business logic (queries)
//! - `local_client/` — ClientHub adapter
//!
//! ### Infrastructure Layer (`infra/`)
//! - `storage/entity/` — SeaORM entities
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

// === PUBLIC API (from SDK) ===
pub use trading_dashboard_sdk::{StatsSummary, TradingDashboardApi, TradingDashboardError};

// === MODULE DEFINITION ===
pub mod module;
pub use module::TradingDashboard;

// === INTERNAL MODULES ===
#[doc(hidden)]
pub mod api;
#[doc(hidden)]
pub mod config;
#[doc(hidden)]
pub mod domain;
#[doc(hidden)]
pub mod infra;
