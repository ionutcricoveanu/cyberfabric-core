use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Kline (candlestick) data
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Kline {
    /// Kline start time (UTC)
    pub start_time: DateTime<Utc>,

    /// Kline close time (UTC)
    pub close_time: DateTime<Utc>,

    /// Trading pair symbol (e.g., "BTCUSDT")
    pub symbol: String,

    /// Kline interval (e.g., "1m", "1h")
    pub interval: String,

    /// Open price
    pub open_price: f64,

    /// Close price
    pub close_price: f64,

    /// High price
    pub high_price: f64,

    /// Low price
    pub low_price: f64,

    /// Base asset volume
    pub base_asset_volume: f64,

    /// Quote asset volume
    pub quote_asset_volume: f64,

    /// Number of trades
    pub number_of_trades: i32,

    /// Taker buy base asset volume
    pub taker_buy_base_asset_volume: f64,

    /// Taker buy quote asset volume
    pub taker_buy_quote_asset_volume: f64,

    /// Whether this kline is closed (complete)
    pub kline_closed: bool,
}

/// Supported kline intervals
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KlineInterval {
    #[serde(rename = "1m")]
    OneMinute,

    #[serde(rename = "5m")]
    FiveMinutes,

    #[serde(rename = "15m")]
    FifteenMinutes,

    #[serde(rename = "1h")]
    OneHour,

    #[serde(rename = "4h")]
    FourHours,
}

impl KlineInterval {
    /// Convert to Binance API string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OneMinute => "1m",
            Self::FiveMinutes => "5m",
            Self::FifteenMinutes => "15m",
            Self::OneHour => "1h",
            Self::FourHours => "4h",
        }
    }

    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "1m" => Some(Self::OneMinute),
            "5m" => Some(Self::FiveMinutes),
            "15m" => Some(Self::FifteenMinutes),
            "1h" => Some(Self::OneHour),
            "4h" => Some(Self::FourHours),
            _ => None,
        }
    }

    /// Get table suffix for database storage
    pub fn table_suffix(&self) -> &'static str {
        self.as_str()
    }
}

/// 24-hour ticker statistics
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Ticker24h {
    /// Trading pair symbol
    pub symbol: String,

    /// Price change
    pub price_change: f64,

    /// Price change percent
    pub price_change_percent: f64,

    /// Weighted average price
    pub weighted_avg_price: f64,

    /// Last price
    pub last_price: f64,

    /// Last quantity
    pub last_qty: f64,

    /// Best bid price
    pub bid_price: f64,

    /// Best ask price
    pub ask_price: f64,

    /// Open price
    pub open_price: f64,

    /// High price
    pub high_price: f64,

    /// Low price
    pub low_price: f64,

    /// Total traded base asset volume
    pub volume: f64,

    /// Total traded quote asset volume
    pub quote_volume: f64,

    /// Statistics open time
    pub open_time: DateTime<Utc>,

    /// Statistics close time
    pub close_time: DateTime<Utc>,

    /// Number of trades
    pub count: i64,
}
