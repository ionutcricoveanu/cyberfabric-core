//! Config Manager Module Implementation
//!
//! Manages trading pair configuration, system settings (YAML), and chart signals.
//!
//! The public API is defined in `config-manager-sdk` and re-exported here.

// === PUBLIC API (from SDK) ===
pub use config_manager_sdk::{ConfigManagerApi, ConfigManagerError, ConfigValue, PairConfig, TradingConfig};

// === MODULE DEFINITION ===
pub mod module;
pub use module::ConfigManager;

// === INTERNAL MODULES ===
#[doc(hidden)]
pub mod api;
#[doc(hidden)]
pub mod config;
#[doc(hidden)]
pub mod domain;
#[doc(hidden)]
pub mod infra;
