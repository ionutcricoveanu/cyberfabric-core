//! REST API DTOs (Data Transfer Objects) for trading-core

use chrono::NaiveDateTime;
use modkit_macros::api_dto;

/// Bot status response DTO
#[api_dto(response)]
#[derive(Debug, Clone)]
pub struct BotStatusResponse {
    pub running: bool,
    pub paused: bool,
    pub open_positions: i64,
    pub last_trade_time: Option<NaiveDateTime>,
}

/// Pause request DTO
#[api_dto(request)]
#[derive(Debug)]
pub struct PauseRequest {
    pub reason: Option<String>,
}

/// Resume request DTO
#[api_dto(request)]
#[derive(Debug)]
pub struct ResumeRequest {
    pub reason: Option<String>,
}

/// Generic action response
#[api_dto(response)]
#[derive(Debug, Clone)]
pub struct ActionResponse {
    pub success: bool,
    pub message: String,
}

/// Trading position info
#[api_dto(response)]
#[derive(Debug, Clone)]
pub struct PositionInfo {
    pub symbol: String,
    pub quantity: f64,
    pub entry_price: f64,
    pub current_price: f64,
    pub pnl: f64,
    pub pnl_pct: f64,
    pub entry_time: NaiveDateTime,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
}

/// Order history entry
#[api_dto(response)]
#[derive(Debug, Clone)]
pub struct OrderHistoryEntry {
    pub id: i64,
    pub symbol: String,
    pub side: String,  // BUY, SELL
    pub quantity: f64,
    pub price: f64,
    pub total_usdt: f64,
    pub timestamp: NaiveDateTime,
    pub status: String,  // FILLED, PENDING, CANCELLED
}

/// Trading summary
#[api_dto(response)]
#[derive(Debug, Clone)]
pub struct TradingSummary {
    pub total_trades: i64,
    pub winning_trades: i64,
    pub losing_trades: i64,
    pub win_rate: f64,
    pub total_pnl: f64,
    pub daily_pnl: f64,
    pub largest_win: f64,
    pub largest_loss: f64,
    pub average_win: f64,
    pub average_loss: f64,
}

/// Health response
#[api_dto(response)]
#[derive(Debug, Clone)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
}
