//! Configuration for the trading-core module

use serde::{Deserialize, Serialize};

/// Configuration for the trading-core module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingCoreConfig {
    /// Binance main database connection string (for buy_orders, sell_orders, pnl, trade_pairs)
    /// Format: "postgresql://user:password@host:port/Binance"
    pub binance_dsn: String,

    /// Binance Klines database connection string (for reading historical data)
    /// Format: "postgresql://user:password@host:port/Binance_Klines"
    pub klines_dsn: String,

    /// Model Data database connection string (for agent decisions, sentiment)
    /// Format: "postgresql://user:password@host:port/Model_Data"
    pub model_data_dsn: String,

    /// Main trading loop interval in seconds (default: 300 = 5 minutes)
    #[serde(default = "default_main_loop_interval")]
    pub main_loop_interval_secs: u64,

    /// Buy strategy execution interval in seconds (default: 1800 = 30 minutes)
    #[serde(default = "default_buy_interval")]
    pub buy_strategy_interval_secs: u64,

    /// Sell strategy execution interval in seconds (default: 900 = 15 minutes)
    #[serde(default = "default_sell_interval")]
    pub sell_strategy_interval_secs: u64,

    /// Default kline interval to use for analysis (1m, 5m, 15m, 1h, 4h)
    #[serde(default = "default_kline_interval")]
    pub kline_interval: String,

    /// Binance API key for authenticated endpoints
    #[serde(default)]
    pub api_key: String,

    /// Binance API secret for HMAC-SHA256 request signing
    #[serde(default)]
    pub api_secret: String,

    /// Enable testnet mode
    #[serde(default)]
    pub testnet: bool,

    /// Position sizing parameters
    #[serde(default)]
    pub position_sizing: PositionSizingConfig,

    /// Risk management parameters
    #[serde(default)]
    pub risk_management: RiskManagementConfig,

    /// ML service gRPC URL
    #[serde(default = "default_ml_service_url")]
    pub ml_service_url: String,

    /// Trading enabled flag (default: true)
    #[serde(default = "default_true")]
    pub trading_enabled: bool,

    /// Buying enabled flag (can be disabled for maintenance, default: true)
    #[serde(default = "default_true")]
    pub buying_enabled: bool,
}

/// Position sizing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionSizingConfig {
    /// Kelly criterion fraction (0.0 to 1.0, default: 0.25 for conservative sizing)
    #[serde(default = "default_kelly_fraction")]
    pub kelly_fraction: f64,

    /// Maximum position size as percentage of portfolio (default: 5.0%)
    #[serde(default = "default_max_position_pct")]
    pub max_position_pct: f64,

    /// Minimum USDT value per position (default: 10.0 USDT)
    #[serde(default = "default_min_position_usdt")]
    pub min_position_usdt: f64,

    /// Fixed position size in USDT (if set, overrides Kelly calculation). Default: None
    #[serde(default)]
    pub fixed_position_usdt: Option<f64>,
}

impl Default for PositionSizingConfig {
    fn default() -> Self {
        Self {
            kelly_fraction: default_kelly_fraction(),
            max_position_pct: default_max_position_pct(),
            min_position_usdt: default_min_position_usdt(),
            fixed_position_usdt: None,
        }
    }
}

/// Risk management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskManagementConfig {
    /// Maximum daily loss limit in USDT (trading stops if exceeded, default: 500.0)
    #[serde(default = "default_max_daily_loss")]
    pub max_daily_loss_usdt: f64,

    /// Stop loss percentage from entry (default: 2.0%)
    #[serde(default = "default_stop_loss_pct")]
    pub stop_loss_pct: f64,

    /// Take profit percentage from entry (default: 5.0%)
    #[serde(default = "default_take_profit_pct")]
    pub take_profit_pct: f64,

    /// Maximum portfolio leverage (default: 1.0 = no leverage)
    #[serde(default = "default_max_leverage")]
    pub max_leverage: f64,

    /// Maximum number of concurrent open positions (default: 5)
    #[serde(default = "default_max_open_positions")]
    pub max_open_positions: i32,

    /// Maximum loss per trade as % of position (default: 100 = close at stop loss)
    #[serde(default = "default_max_loss_per_trade_pct")]
    pub max_loss_per_trade_pct: f64,
}

impl Default for RiskManagementConfig {
    fn default() -> Self {
        Self {
            max_daily_loss_usdt: default_max_daily_loss(),
            stop_loss_pct: default_stop_loss_pct(),
            take_profit_pct: default_take_profit_pct(),
            max_leverage: default_max_leverage(),
            max_open_positions: default_max_open_positions(),
            max_loss_per_trade_pct: default_max_loss_per_trade_pct(),
        }
    }
}

// Default values
fn default_main_loop_interval() -> u64 {
    300 // 5 minutes
}

fn default_buy_interval() -> u64 {
    1800 // 30 minutes
}

fn default_sell_interval() -> u64 {
    900 // 15 minutes
}

fn default_kline_interval() -> String {
    "1h".to_string()
}

fn default_ml_service_url() -> String {
    "http://localhost:50051".to_string()
}

fn default_kelly_fraction() -> f64 {
    0.25
}

fn default_max_position_pct() -> f64 {
    5.0
}

fn default_min_position_usdt() -> f64 {
    10.0
}

fn default_max_daily_loss() -> f64 {
    500.0
}

fn default_stop_loss_pct() -> f64 {
    2.0
}

fn default_take_profit_pct() -> f64 {
    5.0
}

fn default_max_leverage() -> f64 {
    1.0
}

fn default_max_open_positions() -> i32 {
    5
}

fn default_max_loss_per_trade_pct() -> f64 {
    100.0
}

fn default_true() -> bool {
    true
}

impl Default for TradingCoreConfig {
    fn default() -> Self {
        Self {
            binance_dsn: String::new(),
            klines_dsn: String::new(),
            model_data_dsn: String::new(),
            api_key: String::new(),
            api_secret: String::new(),
            main_loop_interval_secs: default_main_loop_interval(),
            buy_strategy_interval_secs: default_buy_interval(),
            sell_strategy_interval_secs: default_sell_interval(),
            kline_interval: default_kline_interval(),
            testnet: false,
            position_sizing: PositionSizingConfig::default(),
            risk_management: RiskManagementConfig::default(),
            ml_service_url: default_ml_service_url(),
            trading_enabled: true,
            buying_enabled: true,
        }
    }
}
