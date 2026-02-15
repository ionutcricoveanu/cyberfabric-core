use std::collections::HashMap;

// ================================================================
// Summary endpoint
// ================================================================

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct IntervalSummaryDto {
    pub interval: String,
    pub total_trainings: i64,
    pub promoted_count: i64,
    pub rejected_count: i64,
    pub avg_promoted_win_rate: Option<f64>,
    pub avg_promoted_return: Option<f64>,
    pub avg_promoted_sharpe: Option<f64>,
    pub avg_promoted_profit_factor: Option<f64>,
    pub promoted_with_sharpe_count: i64,
    pub promoted_with_profit_factor_count: i64,
    pub avg_training_duration: Option<f64>,
    pub unique_symbols: i64,
    pub promotion_rate: f64,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct SummaryTotalsDto {
    pub total_trainings: i64,
    pub total_promoted: i64,
    pub unique_symbols: i64,
    pub promoted_with_sharpe_count: i64,
    pub promoted_with_profit_factor_count: i64,
    pub overall_promotion_rate: f64,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct ModelSummaryResponse {
    pub by_interval: Vec<IntervalSummaryDto>,
    pub totals: SummaryTotalsDto,
}

// ================================================================
// Training history endpoint
// ================================================================

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct TrainingHistoryItemDto {
    pub id: i32,
    pub symbol: String,
    pub interval: String,
    pub trained_at: String,
    pub promoted: bool,
    pub win_rate: Option<f64>,
    pub total_trades: Option<i32>,
    pub total_return: Option<f64>,
    pub sharpe_ratio: Option<f64>,
    pub max_drawdown: Option<f64>,
    pub profit_factor: Option<f64>,
    pub training_duration_seconds: Option<f64>,
    pub backtest_duration_seconds: Option<f64>,
    pub rejection_reason: Option<String>,
    pub wf_median_win_rate: Option<f64>,
    pub wf_mean_return: Option<f64>,
    pub wf_max_drawdown_worst: Option<f64>,
    pub wf_passed_gates: Option<bool>,
    pub oos_win_rate: Option<f64>,
    pub oos_total_trades: Option<i32>,
    pub oos_return: Option<f64>,
    pub oos_max_drawdown: Option<f64>,
    pub oos_passed_gates: Option<bool>,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct TrainingHistoryResponse {
    pub history: Vec<TrainingHistoryItemDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

// ================================================================
// Rejection reasons endpoint
// ================================================================

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct RejectionReasonDto {
    pub reason: String,
    pub count: i64,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct RejectionReasonsResponse {
    pub by_interval: HashMap<String, Vec<RejectionReasonDto>>,
}

// ================================================================
// Symbol status endpoint
// ================================================================

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct SymbolStatusDto {
    pub symbol: String,
    pub interval: String,
    pub trained_at: String,
    pub promoted: bool,
    pub win_rate: Option<f64>,
    pub total_return: Option<f64>,
    pub sharpe_ratio: Option<f64>,
    pub rejection_reason: Option<String>,
    pub model_age_days: Option<f64>,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct SymbolStatusResponse {
    pub symbols: Vec<SymbolStatusDto>,
}

// ================================================================
// Training timeline endpoint
// ================================================================

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct TimelinePointDto {
    pub date: String,
    pub interval: String,
    pub total: i64,
    pub promoted: i64,
    pub avg_win_rate: Option<f64>,
    pub avg_return: Option<f64>,
    pub promotion_rate: f64,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct TrainingTimelineResponse {
    pub timeline: Vec<TimelinePointDto>,
}

// ================================================================
// Performance distribution endpoint
// ================================================================

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct PerformancePointDto {
    pub win_rate: Option<f64>,
    pub total_return: Option<f64>,
    pub sharpe_ratio: Option<f64>,
    pub profit_factor: Option<f64>,
    pub max_drawdown: Option<f64>,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct PerformanceDistributionResponse {
    pub distributions: HashMap<String, Vec<PerformancePointDto>>,
}

// ================================================================
// Calibration metrics endpoint
// ================================================================

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct CalibrationMetricDto {
    pub symbol: String,
    pub model_version: String,
    pub avg_calibration_error: Option<f64>,
    pub predictions: i64,
    pub accuracy: Option<f64>,
    pub avg_confidence: Option<f64>,
    pub avg_confidence_when_correct: Option<f64>,
    pub avg_confidence_when_wrong: Option<f64>,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct CalibrationMetricsResponse {
    pub calibration_metrics: Vec<CalibrationMetricDto>,
}

// ================================================================
// Regime analysis endpoint
// ================================================================

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct RegimeAnalysisDto {
    pub market_regime: String,
    pub regime_conflict: bool,
    pub count: i64,
    pub accuracy: Option<f64>,
    pub avg_confidence: Option<f64>,
    pub avg_calibration_error: Option<f64>,
    pub avg_regime_penalty: Option<f64>,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct RegimeAnalysisResponse {
    pub regime_analysis: Vec<RegimeAnalysisDto>,
}

// ================================================================
// Model age impact endpoint
// ================================================================

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct AgeImpactDto {
    pub age_bucket: String,
    pub predictions: i64,
    pub accuracy: Option<f64>,
    pub avg_calibration_error: Option<f64>,
    pub avg_confidence: Option<f64>,
    pub avg_age_penalty: Option<f64>,
    pub min_age: Option<f64>,
    pub max_age: Option<f64>,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct AgeImpactResponse {
    pub age_impact: Vec<AgeImpactDto>,
}
