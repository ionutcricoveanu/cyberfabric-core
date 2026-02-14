//! Market Intelligence SDK
//!
//! Public API for the `market-intelligence` module:
//! - `MarketIntelligenceApi` trait
//! - Model types for agent decisions
//! - Error type (`MarketIntelligenceError`)

#![forbid(unsafe_code)]
#![deny(rust_2018_idioms)]

pub mod client;
pub mod errors;
pub mod models;

pub use client::MarketIntelligenceApi;
pub use errors::MarketIntelligenceError;
pub use models::AgentDecision;
