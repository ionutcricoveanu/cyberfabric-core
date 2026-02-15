// ================================================================
// Overview endpoint
// ================================================================

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct PerformanceOverviewResponse {
    pub total_trades: i64,
    pub winning_trades: i64,
    pub losing_trades: i64,
    pub win_rate: f64,
    pub avg_profit: f64,
    pub max_profit: f64,
    pub max_loss: f64,
    pub profit_factor: f64,
    pub ml_trades: i64,
    pub signal_trades: i64,
    pub ml_percentage: f64,
    pub signal_percentage: f64,
    pub total_profit_usdc: f64,
    pub time_period_hours: i64,
}

// ================================================================
// Strategy comparison endpoint
// ================================================================

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct StrategyDataDto {
    pub strategy_name: String,
    pub total_trades: i64,
    pub winning_trades: i64,
    pub win_rate: f64,
    pub avg_profit: f64,
    pub total_profit_usdc: f64,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct ComparisonDto {
    pub win_rate_delta: f64,
    pub avg_profit_delta: f64,
    pub better_strategy: String,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct StrategyComparisonResponse {
    pub ml_strategy: StrategyDataDto,
    pub signal_strategy: StrategyDataDto,
    pub comparison: ComparisonDto,
    pub time_period_hours: i64,
}

// ================================================================
// Alerts endpoint
// ================================================================

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct AlertDto {
    pub severity: String,
    pub alert_type: String,
    pub message: String,
    pub value: f64,
    pub threshold: f64,
    pub recommendation: String,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct AlertsResponse {
    pub alert_count: i64,
    pub alerts: Vec<AlertDto>,
    pub timestamp: String,
}
