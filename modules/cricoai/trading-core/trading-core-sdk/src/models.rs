//! Domain models for the trading-core module.
//!
//! Transport-agnostic (no serde, no HTTP types).

/// Current bot operational status.
pub struct BotStatus {
    pub running: bool,
    pub paused: bool,
    pub open_positions: i64,
    pub last_trade_time: Option<chrono::NaiveDateTime>,
}
