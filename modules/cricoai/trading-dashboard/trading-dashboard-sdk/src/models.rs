//! Domain models for the trading-dashboard module.
//!
//! These types are transport-agnostic (no serde, no HTTP types).
//! REST DTOs live in the module's `api/rest/dto.rs`.

/// Aggregated trading statistics summary.
pub struct StatsSummary {
    pub total_trades: i64,
    pub open_orders: i64,
    pub total_pnl: f64,
    pub total_asset_value: f64,
    pub available_usdc: f64,
    pub trading_enabled: bool,
}

/// Single P&L record for a day.
pub struct DailyPnl {
    pub date: chrono::NaiveDateTime,
    pub pnl: f64,
    pub client_order_id: Option<String>,
}

/// A completed trade (buy or sell).
pub struct TradeRecord {
    pub id: i32,
    pub symbol: Option<String>,
    pub side: Option<String>,
    pub price: Option<f64>,
    pub qty: Option<f64>,
    pub pnl: Option<f64>,
    pub status: Option<String>,
    pub transact_time: Option<chrono::NaiveDateTime>,
}
