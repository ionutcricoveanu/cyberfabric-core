//! Domain models for the config-manager module.
//!
//! Transport-agnostic (no serde, no HTTP types).

use std::collections::HashMap;

/// Structured trading configuration read from YAML.
pub struct TradingConfig {
    pub settings: HashMap<String, ConfigValue>,
    pub environment: String,
}

/// A loosely-typed config value (bool, int, float, string).
#[derive(Debug, Clone)]
pub enum ConfigValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
}

/// Result of a config update.
pub struct ConfigUpdateResult {
    pub updated_keys: Vec<String>,
}

/// Raw YAML config content.
pub struct RawConfig {
    pub yaml: String,
    pub environment: String,
}

/// A trading pair with price evolution and order data.
pub struct PairConfig {
    pub pair: String,
    pub change_24h: f64,
    pub excluded: bool,
    pub highest_percent: f64,
    pub highest_price: f64,
    pub current_price: f64,
    pub lowest_price: f64,
    pub lowest_percent: f64,
    pub open_orders: i64,
    pub last_purchase_time: Option<String>,
    pub last_sell_time: Option<String>,
    pub est_profit_percent: f64,
}

/// Editable trading settings (flat key-value from YAML).
pub struct TradingSettings {
    pub trade_interval_secs: i64,
    pub max_open_orders: i32,
    pub default_profit_target: f64,
    pub default_stop_loss: f64,
}

/// Result of a container restart.
pub struct RestartResult {
    pub container: String,
    pub environment: String,
}
