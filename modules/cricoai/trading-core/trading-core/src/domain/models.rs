//! Internal domain models

use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Buy decision from ML service or agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuySignal {
    pub symbol: String,
    pub action: String,  // BUY, HOLD, SELL
    pub confidence: f64,
    pub predicted_change_pct: f64,
    pub timestamp: NaiveDateTime,
}

/// Sell decision for managing exits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SellSignal {
    pub symbol: String,
    pub action: String,  // BUY, HOLD, SELL
    pub confidence: f64,
    pub pnl_pct: f64,
    pub timestamp: NaiveDateTime,
}

/// Trading position
#[derive(Debug, Clone)]
pub struct Position {
    pub symbol: String,
    pub quantity: f64,
    pub entry_price: Decimal,
    pub entry_time: NaiveDateTime,
    pub stop_loss: Option<Decimal>,
    pub take_profit: Option<Decimal>,
    pub id: Option<i64>,
}

/// Order to be placed
#[derive(Debug, Clone)]
pub struct Order {
    pub symbol: String,
    pub side: String,  // BUY, SELL
    pub quantity: f64,
    pub price: Decimal,
    pub order_type: String,  // LIMIT, MARKET
    pub time_in_force: String,  // GTC, IOC, FOK
}

/// Quote from pricing service
#[derive(Debug, Clone)]
pub struct Quote {
    pub symbol: String,
    pub bid: Decimal,
    pub ask: Decimal,
    pub last_trade_price: Decimal,
    pub timestamp: NaiveDateTime,
}

/// Account balance snapshot
#[derive(Debug, Clone)]
pub struct AccountBalance {
    pub asset: String,
    pub free: Decimal,
    pub locked: Decimal,
    pub total: Decimal,
}

/// Trade summary for reporting
#[derive(Debug, Clone)]
pub struct TradeSummary {
    pub symbol: String,
    pub side: String,  // BUY, SELL
    pub quantity: f64,
    pub price: Decimal,
    pub total: Decimal,
    pub pnl: Option<Decimal>,
    pub pnl_pct: Option<f64>,
    pub timestamp: NaiveDateTime,
}

/// Risk metrics
#[derive(Debug, Clone)]
pub struct RiskMetrics {
    pub open_positions: i32,
    pub total_exposure: Decimal,
    pub daily_pnl: Decimal,
    pub max_daily_loss: Decimal,
    pub current_leverage: f64,
    pub portfolio_heat: f64,  // % of balance at risk
}

/// Market regime indicator
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MarketRegime {
    Bullish,
    Bearish,
    Ranging,
    Volatile,
}

impl std::fmt::Display for MarketRegime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MarketRegime::Bullish => write!(f, "BULLISH"),
            MarketRegime::Bearish => write!(f, "BEARISH"),
            MarketRegime::Ranging => write!(f, "RANGING"),
            MarketRegime::Volatile => write!(f, "VOLATILE"),
        }
    }
}
