use trading_dashboard_sdk::models::StatsSummary;

/// REST DTO for trading statistics summary.
#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct StatsSummaryDto {
    pub total_trades: i64,
    pub open_orders: i64,
    pub total_pnl: f64,
    pub total_asset_value: f64,
    pub available_usdc: f64,
    pub trading_enabled: bool,
}

impl From<StatsSummary> for StatsSummaryDto {
    fn from(s: StatsSummary) -> Self {
        Self {
            total_trades: s.total_trades,
            open_orders: s.open_orders,
            total_pnl: s.total_pnl,
            total_asset_value: s.total_asset_value,
            available_usdc: s.available_usdc,
            trading_enabled: s.trading_enabled,
        }
    }
}

/// Daily P&L aggregation.
#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct DailyPnlDto {
    pub date: String,
    pub trades: i64,
    pub wins: i64,
    pub losses: i64,
    pub daily_pnl: f64,
}

/// Wrapper for daily P&L list.
#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct DailyPnlListDto {
    pub daily_pnl: Vec<DailyPnlDto>,
}

/// A recent trade (completed sell order with P&L).
#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct RecentTradeDto {
    pub timestamp: Option<String>,
    pub symbol: Option<String>,
    pub pnl: Option<f64>,
    pub client_order_id: Option<String>,
}

/// Wrapper for recent trades list.
#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct RecentTradesListDto {
    pub trades: Vec<RecentTradeDto>,
}

/// Aggregate trading statistics.
#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct TradingStatisticsDto {
    pub total_pnl: f64,
    pub number_of_trades: i64,
    pub unique_days: i64,
    pub avg_pnl_per_trade: f64,
    pub avg_pnl_per_day: f64,
    pub trades_per_day: f64,
    pub available_usdc: f64,
    pub available_bnb: f64,
    pub pairs_evolution_day: PairsEvolutionDto,
    pub pairs_evolution_hour: PairsEvolutionDto,
}

/// Pair evolution counts (positive/negative).
#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct PairsEvolutionDto {
    pub positive: i64,
    pub negative: i64,
}

/// Top trading pair by P&L.
#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct TopPairDto {
    pub symbol: String,
    pub total_trades: i64,
    pub wins: i64,
    pub losses: i64,
    pub total_pnl: f64,
    pub win_rate: f64,
}

/// Wrapper for top pairs list.
#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct TopPairsListDto {
    pub pairs: Vec<TopPairDto>,
}

/// Hourly asset value data point.
#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct AssetValuePointDto {
    pub date: String,
    pub value: f64,
}

/// Total assets value response with 24h hourly data.
#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct TotalAssetsValueDto {
    pub hourly_24h: Vec<AssetValuePointDto>,
}

/// Open buy order.
#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct OpenOrderDto {
    pub id: i32,
    pub symbol: Option<String>,
    pub client_order_id: Option<String>,
    pub transact_time: Option<String>,
    pub price: Option<f64>,
    pub qty: Option<f64>,
    pub status: Option<String>,
    pub est_profit_percent: Option<f64>,
    pub highest_percentage_since_buy: Option<f64>,
    pub lowest_percentage_since_buy: Option<f64>,
}

/// Wrapper for open orders list.
#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct OpenOrdersListDto {
    pub orders: Vec<OpenOrderDto>,
}

/// Account balance overview.
#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct AccountBalancesDto {
    pub available_usdc: f64,
    pub total_asset_val: f64,
    pub total_value: f64,
    pub invested: f64,
    pub profit_loss: f64,
    pub profit_loss_percent: f64,
}

// ============================================================
// Trading Orders — pairs-evolution, excluded
// ============================================================

/// A pair's price evolution data.
#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct PairEvolutionItemDto {
    pub pair: String,
    pub highest_price: Option<f64>,
    pub current_price: Option<f64>,
    pub lowest_price: Option<f64>,
    pub highest_percentage: Option<f64>,
    pub lowest_percentage: Option<f64>,
    pub one_day_change_percentage: Option<f64>,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct PairsEvolutionListDto {
    pub pairs: Vec<PairEvolutionItemDto>,
}

/// An excluded trading pair.
#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct ExcludedPairDto {
    pub id: i32,
    pub pair: Option<String>,
    pub added_date: Option<String>,
    pub auto_remove: Option<bool>,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct ExcludedPairsListDto {
    pub pairs: Vec<ExcludedPairDto>,
}

// ============================================================
// Models — summary, training-history, rejection-reasons, etc.
// ============================================================

/// Model training summary.
#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct ModelsSummaryDto {
    pub total_models_trained: i64,
    pub promoted_models: i64,
    pub rejected_models: i64,
    pub promotion_rate: f64,
    pub unique_symbols: i64,
    pub avg_win_rate: f64,
    pub avg_sharpe_ratio: f64,
}

/// A single model training history record.
#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct TrainingHistoryDto {
    pub id: i32,
    pub symbol: String,
    pub trained_at: String,
    pub promoted: bool,
    pub win_rate: Option<f64>,
    pub total_trades: Option<i32>,
    pub total_return: Option<f64>,
    pub sharpe_ratio: Option<f64>,
    pub max_drawdown: Option<f64>,
    pub profit_factor: Option<f64>,
    pub training_duration_seconds: Option<f64>,
    pub rejection_reason: Option<String>,
    pub interval: Option<String>,
    pub wf_passed_gates: Option<bool>,
    pub oos_passed_gates: Option<bool>,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct TrainingHistoryListDto {
    pub records: Vec<TrainingHistoryDto>,
}

/// Rejection reason aggregation.
#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct RejectionReasonDto {
    pub reason: String,
    pub count: i64,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct RejectionReasonsListDto {
    pub reasons: Vec<RejectionReasonDto>,
}

/// Symbol model status.
#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct SymbolModelStatusDto {
    pub symbol: String,
    pub total_trained: i64,
    pub promoted: i64,
    pub latest_trained_at: Option<String>,
    pub latest_promoted: Option<bool>,
    pub latest_win_rate: Option<f64>,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct SymbolModelStatusListDto {
    pub symbols: Vec<SymbolModelStatusDto>,
}

/// Training timeline data point.
#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct TrainingTimelineDto {
    pub date: String,
    pub total_trained: i64,
    pub promoted: i64,
    pub rejected: i64,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct TrainingTimelineListDto {
    pub timeline: Vec<TrainingTimelineDto>,
}

/// Performance distribution bucket.
#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct PerformanceDistributionDto {
    pub win_rate_buckets: Vec<BucketDto>,
    pub sharpe_ratio_buckets: Vec<BucketDto>,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct BucketDto {
    pub range: String,
    pub count: i64,
}

/// Calibration metrics summary.
#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct CalibrationMetricsSummaryDto {
    pub total_predictions: i64,
    pub direction_accuracy: f64,
    pub avg_calibration_error: f64,
    pub avg_confidence: f64,
    pub by_regime: Vec<RegimeCalibrationDto>,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct RegimeCalibrationDto {
    pub regime: String,
    pub count: i64,
    pub accuracy: f64,
    pub avg_confidence: f64,
}

/// Regime analysis.
#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct RegimeAnalysisDto {
    pub regime: String,
    pub count: i64,
    pub promoted: i64,
    pub avg_win_rate: f64,
    pub avg_sharpe: f64,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct RegimeAnalysisListDto {
    pub regimes: Vec<RegimeAnalysisDto>,
}

/// Model age impact analysis.
#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct ModelAgeImpactDto {
    pub age_bucket: String,
    pub count: i64,
    pub direction_accuracy: f64,
    pub avg_quality_score: f64,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct ModelAgeImpactListDto {
    pub buckets: Vec<ModelAgeImpactDto>,
}

// ============================================================
// Predictions
// ============================================================

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct PredictionDto {
    pub id: i32,
    pub symbol: Option<String>,
    pub prediction_timestamp: Option<String>,
    pub model_version: Option<String>,
    pub market_regime: Option<String>,
    pub up_probability: Option<f64>,
    pub down_probability: Option<f64>,
    pub confidence_score: Option<f64>,
    pub predicted_profit_percent: Option<f64>,
    pub resulted_in_trade: Option<bool>,
    pub direction_correct: Option<bool>,
    pub actual_profit_percent: Option<f64>,
    pub prediction_quality_score: Option<f64>,
    pub model_age_days: Option<i32>,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct PredictionsListDto {
    pub predictions: Vec<PredictionDto>,
}

// ============================================================
// Agents — overview, decisions, sentiment, performance, llm, config
// ============================================================

/// Agent overview (aggregate per agent type).
#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct AgentOverviewDto {
    pub agent_type: String,
    pub total_decisions: i64,
    pub recent_decisions_24h: i64,
    pub avg_confidence: f64,
    pub total_cost_usd: f64,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct AgentOverviewListDto {
    pub agents: Vec<AgentOverviewDto>,
}

/// Agent decision record.
#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct AgentDecisionDto {
    pub id: i32,
    pub agent_type: String,
    pub decision_type: String,
    pub symbol: Option<String>,
    pub recommendation: Option<String>,
    pub confidence: Option<f64>,
    pub status: Option<String>,
    pub created_at: Option<String>,
    pub processing_time_ms: Option<i32>,
    pub cost_usd: Option<f64>,
    pub llm_model: Option<String>,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct AgentDecisionsListDto {
    pub decisions: Vec<AgentDecisionDto>,
}

/// Agent decisions timeline (daily aggregate).
#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct AgentDecisionTimelineDto {
    pub date: String,
    pub total: i64,
    pub applied: i64,
    pub rejected: i64,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct AgentDecisionTimelineListDto {
    pub timeline: Vec<AgentDecisionTimelineDto>,
}

/// Sentiment trend data point.
#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct SentimentTrendDto {
    pub date: String,
    pub avg_score: f64,
    pub count: i64,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct SentimentTrendsListDto {
    pub trends: Vec<SentimentTrendDto>,
}

/// Agent performance metrics.
#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct AgentPerformanceDto {
    pub id: i32,
    pub agent_type: String,
    pub date: String,
    pub decisions_made: Option<i32>,
    pub decisions_applied: Option<i32>,
    pub decisions_rejected: Option<i32>,
    pub avg_confidence: Option<f64>,
    pub avg_processing_time_ms: Option<i32>,
    pub total_cost_usd: Option<f64>,
    pub positive_outcomes: Option<i32>,
    pub negative_outcomes: Option<i32>,
    pub neutral_outcomes: Option<i32>,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct AgentPerformanceListDto {
    pub metrics: Vec<AgentPerformanceDto>,
}

/// LLM usage record.
#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct LlmUsageDto {
    pub total_requests: i64,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub total_cost_usd: f64,
    pub avg_processing_time_ms: f64,
    pub by_model: Vec<LlmUsageByModelDto>,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct LlmUsageByModelDto {
    pub model: String,
    pub requests: i64,
    pub total_tokens: i64,
    pub cost_usd: f64,
}

/// Agent config record.
#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct AgentConfigDto {
    pub id: i32,
    pub agent_type: String,
    pub enabled: bool,
    pub mode: Option<String>,
    pub decision_frequency_minutes: Option<i32>,
    pub confidence_threshold: Option<f64>,
    pub llm_provider: Option<String>,
    pub llm_model: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct AgentConfigListDto {
    pub configs: Vec<AgentConfigDto>,
}

// ============================================================
// Agent Impact — buy-impact, sell-impact, effectiveness
// ============================================================

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct AgentTradeImpactDto {
    pub id: i32,
    pub pair: String,
    pub agent_recommendation: Option<String>,
    pub recommendation_confidence: Option<f64>,
    pub action_taken: Option<String>,
    pub entry_price: Option<f64>,
    pub exit_price: Option<f64>,
    pub profit_pct: Option<f64>,
    pub counterfactual_profit_pct: Option<f64>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct AgentTradeImpactListDto {
    pub impacts: Vec<AgentTradeImpactDto>,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct AgentEffectivenessDto {
    pub total_trades: i64,
    pub agent_influenced: i64,
    pub avg_profit_with_agent: f64,
    pub avg_profit_without_agent: f64,
    pub agent_value_add: f64,
}

// ============================================================
// Performance — overview, strategy-comparison, alerts
// ============================================================

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct PerformanceOverviewDto {
    pub total_pnl: f64,
    pub win_rate: f64,
    pub total_trades: i64,
    pub best_day_pnl: f64,
    pub worst_day_pnl: f64,
    pub avg_daily_pnl: f64,
    pub max_drawdown: f64,
    pub current_streak: i64,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct StrategyComparisonDto {
    pub strategy: String,
    pub trades: i64,
    pub win_rate: f64,
    pub total_pnl: f64,
    pub avg_pnl: f64,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct StrategyComparisonListDto {
    pub strategies: Vec<StrategyComparisonDto>,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct PerformanceAlertDto {
    pub alert_type: String,
    pub message: String,
    pub severity: String,
    pub timestamp: String,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct PerformanceAlertsListDto {
    pub alerts: Vec<PerformanceAlertDto>,
}

// ============================================================
// DB Tables
// ============================================================

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct DbTableDto {
    pub table_name: String,
    pub row_count: i64,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct DbTablesListDto {
    pub tables: Vec<DbTableDto>,
}
