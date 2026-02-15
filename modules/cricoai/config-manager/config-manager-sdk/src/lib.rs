//! Config Manager SDK
//!
//! Public API for the `config-manager` module:
//! - `ConfigManagerApi` trait
//! - Model types for trading pair config and settings
//! - Error type (`ConfigManagerError`)

#![forbid(unsafe_code)]
#![deny(rust_2018_idioms)]

pub mod client;
pub mod errors;
pub mod models;

pub use client::ConfigManagerApi;
pub use errors::ConfigManagerError;
pub use models::{ConfigValue, PairConfig, TradingConfig, TradingSettings};
