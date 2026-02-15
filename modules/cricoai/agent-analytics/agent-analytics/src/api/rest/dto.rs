// ================================================================
// Overview endpoint
// ================================================================

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct AgentSummaryDto {
    pub total_decisions_today: i64,
    pub decisions_pending: i64,
    pub decisions_applied: i64,
    pub decisions_rejected: i64,
    pub avg_confidence: Option<f64>,
    pub total_cost_today: Option<f64>,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct AgentInfoDto {
    pub agent_type: String,
    pub enabled: bool,
    pub mode: Option<String>,
    pub llm_provider: Option<String>,
    pub llm_model: Option<String>,
    pub decisions_today: i64,
    pub avg_confidence_today: Option<f64>,
    pub cost_today: Option<f64>,
    pub positive_outcomes_today: i64,
    pub success_rate_today: Option<f64>,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct AgentOverviewResponse {
    pub summary: AgentSummaryDto,
    pub agents: Vec<AgentInfoDto>,
}

// ================================================================
// Recent decisions endpoint
// ================================================================

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct DecisionDto {
    pub id: i64,
    pub agent_type: String,
    pub decision_type: Option<String>,
    pub target_service: Option<String>,
    pub symbol: Option<String>,
    pub recommendation: Option<String>,
    pub reasoning: Option<String>,
    pub confidence: Option<f64>,
    pub action_data: Option<serde_json::Value>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub created_at: Option<String>,
    pub processed_at: Option<String>,
    pub llm_model: Option<String>,
    pub processing_time_ms: Option<f64>,
    pub cost_usd: Option<f64>,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct RecentDecisionsResponse {
    pub decisions: Vec<DecisionDto>,
}

// ================================================================
// Decisions timeline endpoint
// ================================================================

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct TimelinePointDto {
    pub hour: String,
    pub decisions_count: i64,
    pub avg_confidence: Option<f64>,
    pub pending: i64,
    pub applied: i64,
    pub rejected: i64,
    pub acknowledged: i64,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct DecisionsTimelineResponse {
    pub timeline: Vec<TimelinePointDto>,
}

// ================================================================
// Sentiment trends endpoint
// ================================================================

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct SentimentDataDto {
    pub id: i64,
    pub symbol: String,
    pub sentiment_score: Option<f64>,
    pub sentiment_source: Option<String>,
    pub positive_signals: Option<serde_json::Value>,
    pub negative_signals: Option<serde_json::Value>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct SentimentAggDto {
    pub hour: String,
    pub avg_sentiment: Option<f64>,
    pub symbols_analyzed: i64,
    pub total_readings: i64,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct SentimentTrendsResponse {
    pub sentiment_data: Vec<SentimentDataDto>,
    pub aggregated_by_hour: Vec<SentimentAggDto>,
}

// ================================================================
// Performance metrics endpoint
// ================================================================

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct PerformanceMetricDto {
    pub agent_type: String,
    pub date: String,
    pub decisions_made: i64,
    pub decisions_applied: i64,
    pub decisions_rejected: i64,
    pub avg_confidence: Option<f64>,
    pub avg_processing_time_ms: Option<f64>,
    pub total_cost_usd: Option<f64>,
    pub positive_outcomes: i64,
    pub negative_outcomes: i64,
    pub neutral_outcomes: i64,
    pub success_rate: f64,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct PerformanceMetricsResponse {
    pub performance: Vec<PerformanceMetricDto>,
}

// ================================================================
// LLM usage endpoint
// ================================================================

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct UsageByDayDto {
    pub date: String,
    pub total_requests: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub total_cost_usd: Option<f64>,
    pub avg_processing_time_ms: Option<f64>,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct UsageByAgentDto {
    pub agent_type: String,
    pub total_requests: i64,
    pub total_cost_usd: Option<f64>,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct UsageByModelDto {
    pub llm_model: String,
    pub total_requests: i64,
    pub total_cost_usd: Option<f64>,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct LlmUsageResponse {
    pub usage_by_day: Vec<UsageByDayDto>,
    pub usage_by_agent: Vec<UsageByAgentDto>,
    pub usage_by_model: Vec<UsageByModelDto>,
}

// ================================================================
// Agent config endpoint
// ================================================================

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct AgentConfigDto {
    pub agent_type: String,
    pub enabled: bool,
    pub mode: Option<String>,
    pub decision_frequency_minutes: Option<i32>,
    pub confidence_threshold: Option<f64>,
    pub config: Option<serde_json::Value>,
    pub llm_provider: Option<String>,
    pub llm_model: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct AgentConfigResponse {
    pub agents: Vec<AgentConfigDto>,
}

// ================================================================
// Strategy experiments endpoint
// ================================================================

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct ExperimentDto {
    pub id: i64,
    pub strategy_name: String,
    pub description: Option<String>,
    pub parameters: Option<serde_json::Value>,
    pub status: Option<String>,
    pub created_by: Option<String>,
    pub backtest_results: Option<serde_json::Value>,
    pub performance_metrics: Option<serde_json::Value>,
    pub pair: Option<String>,
    pub baseline_comparison: Option<serde_json::Value>,
    pub improvement_pct: Option<f64>,
    pub promoted_at: Option<String>,
    pub retired_at: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct StatusCountDto {
    pub status: String,
    pub count: i64,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct StrategyExperimentsResponse {
    pub experiments: Vec<ExperimentDto>,
    pub status_counts: Vec<StatusCountDto>,
}

// ================================================================
// Strategy leaderboard endpoint
// ================================================================

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct LeaderboardEntryDto {
    pub id: i64,
    pub strategy_name: String,
    pub pair: Option<String>,
    pub win_rate: Option<f64>,
    pub sharpe_ratio: Option<f64>,
    pub total_pnl: Option<f64>,
    pub total_trades: Option<i64>,
    pub avg_trade_duration_minutes: Option<f64>,
    pub max_drawdown: Option<f64>,
    pub profit_factor: Option<f64>,
    pub avg_return_pct: Option<f64>,
    pub last_evaluated: Option<String>,
    pub rank: Option<i32>,
    pub score: Option<f64>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct StrategyLeaderboardResponse {
    pub leaderboard: Vec<LeaderboardEntryDto>,
}

// ================================================================
// Indicator analysis endpoint
// ================================================================

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct IndicatorDto {
    pub id: i64,
    pub indicator_name: String,
    pub importance_score: Option<f64>,
    pub correlation_with_profit: Option<f64>,
    pub optimal_params: Option<serde_json::Value>,
    pub combination_group: Option<String>,
    pub pair: Option<String>,
    pub sample_size: Option<i64>,
    pub evaluation_period_days: Option<i32>,
    pub notes: Option<String>,
    pub evaluated_at: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct IndicatorAnalysisResponse {
    pub indicators: Vec<IndicatorDto>,
}
