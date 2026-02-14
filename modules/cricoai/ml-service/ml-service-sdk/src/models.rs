//! Domain models for the ML service.
//!
//! Transport-agnostic (no serde, no HTTP types).

/// ML model prediction for a trading symbol.
pub struct Prediction {
    pub symbol: String,
    pub interval: String,
    pub direction: String,
    pub confidence: f64,
    pub predicted_change_pct: f64,
}

/// Technical indicator values for a symbol.
pub struct TechnicalIndicators {
    pub symbol: String,
    pub rsi: f64,
    pub macd: f64,
    pub macd_signal: f64,
    pub bollinger_upper: f64,
    pub bollinger_lower: f64,
    pub ema_12: f64,
    pub ema_26: f64,
}

/// Market regime classification.
pub struct MarketRegime {
    pub symbol: String,
    pub regime: String,
    pub confidence: f64,
}
