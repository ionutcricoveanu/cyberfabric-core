//! Domain models for the market-data module.
//!
//! Transport-agnostic (no serde, no HTTP types).

/// Latest price for a trading symbol.
pub struct Price {
    pub symbol: String,
    pub price: f64,
    pub timestamp: chrono::NaiveDateTime,
}

/// OHLCV candlestick data point.
pub struct Kline {
    pub symbol: String,
    pub interval: String,
    pub open_time: chrono::NaiveDateTime,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}
