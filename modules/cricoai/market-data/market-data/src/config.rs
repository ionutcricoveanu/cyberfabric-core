use serde::{Deserialize, Serialize};

/// Configuration for the market-data module
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MarketDataConfig {
    /// Binance Klines database connection string
    /// Format: "postgresql://user:password@host:port/Binance_Klines"
    pub klines_dsn: String,

    /// Binance main database connection string (for reading trade_pairs)
    /// Format: "postgresql://user:password@host:port/Binance"
    pub binance_dsn: String,

    /// Binance API base URL (default: https://api.binance.com)
    #[serde(default = "default_binance_url")]
    pub binance_url: String,

    /// Kline intervals to collect (comma-separated: 1m,5m,15m,1h,4h)
    #[serde(default = "default_intervals")]
    pub intervals: String,

    /// Collection interval in seconds (how often to poll Binance)
    #[serde(default = "default_collection_interval")]
    pub collection_interval_secs: u64,

    /// Maximum number of klines to fetch per request (default: 500)
    #[serde(default = "default_limit")]
    pub limit: u32,

    /// Enable testnet mode
    #[serde(default)]
    pub testnet: bool,
}

fn default_binance_url() -> String {
    "https://api.binance.com".to_string()
}

fn default_intervals() -> String {
    "1m,5m,15m,1h,4h".to_string()
}

fn default_collection_interval() -> u64 {
    60 // Poll every 60 seconds
}

fn default_limit() -> u32 {
    500
}

impl MarketDataConfig {
    /// Parse the intervals string into a vector
    pub fn parse_intervals(&self) -> Vec<String> {
        self.intervals
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }
}
