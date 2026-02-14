//! Domain models for the config-manager module.
//!
//! Transport-agnostic (no serde, no HTTP types).

/// Configuration for a single trading pair.
pub struct PairConfig {
    pub symbol: String,
    pub profit_target: f64,
    pub stop_loss: f64,
    pub enabled: bool,
}

/// Editable trading settings.
pub struct TradingSettings {
    pub trade_interval_secs: i64,
    pub max_open_orders: i32,
    pub default_profit_target: f64,
    pub default_stop_loss: f64,
}
