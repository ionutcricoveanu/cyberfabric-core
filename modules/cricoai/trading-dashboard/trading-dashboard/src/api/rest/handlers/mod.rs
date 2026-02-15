use axum::Extension;
use axum::extract::Query;
use modkit::api::prelude::*;
use modkit_security::SecurityContext;
use serde::Deserialize;
use std::sync::Arc;
use tracing::info;

use crate::api::rest::dto::{
    AccountBalancesDto, AgentConfigListDto, AgentDecisionTimelineListDto,
    AgentDecisionsListDto, AgentEffectivenessDto, AgentOverviewListDto,
    AgentPerformanceListDto, AgentTradeImpactListDto,
    CalibrationMetricsSummaryDto, DailyPnlListDto, DbTablesListDto,
    ExcludedPairsListDto, LlmUsageDto, ModelAgeImpactListDto, ModelsSummaryDto,
    OpenOrdersListDto, PairsEvolutionListDto, PerformanceAlertsListDto,
    PerformanceDistributionDto, PerformanceOverviewDto, PredictionsListDto,
    RecentTradesListDto, RegimeAnalysisListDto, RejectionReasonsListDto,
    SentimentTrendsListDto, StatsSummaryDto, StrategyComparisonListDto,
    SymbolModelStatusListDto, TopPairsListDto, TotalAssetsValueDto,
    TrainingHistoryListDto, TrainingTimelineListDto, TradingStatisticsDto,
};
use crate::api::rest::error::to_problem;
use crate::domain::service::TradingDashboardService;

#[derive(Debug, Deserialize)]
pub struct StatsQuery {
    #[serde(default = "default_time_range")]
    pub time_range: String,
}

fn default_time_range() -> String {
    "24h".to_string()
}

#[derive(Debug, Deserialize)]
pub struct LimitQuery {
    #[serde(default = "default_limit")]
    pub limit: u64,
}

fn default_limit() -> u64 {
    50
}

pub(crate) async fn get_stats_summary(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<TradingDashboardService>>,
    Query(query): Query<StatsQuery>,
) -> ApiResult<JsonBody<StatsSummaryDto>> {
    info!(subject_id = %ctx.subject_id(), "GET /trading-dashboard/v1/stats/summary");
    let summary = svc
        .get_stats_summary(&ctx, &query.time_range)
        .await
        .map_err(to_problem)?;
    Ok(Json(StatsSummaryDto::from(summary)))
}

pub(crate) async fn get_pnl_by_day(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<TradingDashboardService>>,
) -> ApiResult<JsonBody<DailyPnlListDto>> {
    info!(subject_id = %ctx.subject_id(), "GET /trading-dashboard/v1/stats/pnl-by-day");
    let result = svc.get_pnl_by_day(&ctx).await.map_err(to_problem)?;
    Ok(Json(result))
}

pub(crate) async fn get_recent_trades(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<TradingDashboardService>>,
    Query(query): Query<LimitQuery>,
) -> ApiResult<JsonBody<RecentTradesListDto>> {
    info!(subject_id = %ctx.subject_id(), "GET /trading-dashboard/v1/stats/recent-trades");
    let result = svc.get_recent_trades(&ctx, query.limit).await.map_err(to_problem)?;
    Ok(Json(result))
}

pub(crate) async fn get_statistics(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<TradingDashboardService>>,
) -> ApiResult<JsonBody<TradingStatisticsDto>> {
    info!(subject_id = %ctx.subject_id(), "GET /trading-dashboard/v1/stats/statistics");
    let result = svc.get_statistics(&ctx).await.map_err(to_problem)?;
    Ok(Json(result))
}

pub(crate) async fn get_top_pairs(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<TradingDashboardService>>,
) -> ApiResult<JsonBody<TopPairsListDto>> {
    info!(subject_id = %ctx.subject_id(), "GET /trading-dashboard/v1/stats/top-pairs");
    let result = svc.get_top_pairs(&ctx).await.map_err(to_problem)?;
    Ok(Json(result))
}

pub(crate) async fn get_total_assets_value(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<TradingDashboardService>>,
) -> ApiResult<JsonBody<TotalAssetsValueDto>> {
    info!(subject_id = %ctx.subject_id(), "GET /trading-dashboard/v1/stats/total-assets-value");
    let result = svc.get_total_assets_value(&ctx).await.map_err(to_problem)?;
    Ok(Json(result))
}

pub(crate) async fn get_open_orders(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<TradingDashboardService>>,
) -> ApiResult<JsonBody<OpenOrdersListDto>> {
    info!(subject_id = %ctx.subject_id(), "GET /trading-dashboard/v1/orders/open");
    let result = svc.get_open_orders(&ctx).await.map_err(to_problem)?;
    Ok(Json(result))
}

pub(crate) async fn get_account_balances(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<TradingDashboardService>>,
) -> ApiResult<JsonBody<AccountBalancesDto>> {
    info!(subject_id = %ctx.subject_id(), "GET /trading-dashboard/v1/account/balances");
    let result = svc.get_account_balances(&ctx).await.map_err(to_problem)?;
    Ok(Json(result))
}

// ================================================================
// Trading Orders
// ================================================================

pub(crate) async fn get_pairs_evolution(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<TradingDashboardService>>,
) -> ApiResult<JsonBody<PairsEvolutionListDto>> {
    let result = svc.get_pairs_evolution(&ctx).await.map_err(to_problem)?;
    Ok(Json(result))
}

pub(crate) async fn get_excluded_pairs(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<TradingDashboardService>>,
) -> ApiResult<JsonBody<ExcludedPairsListDto>> {
    let result = svc.get_excluded_pairs(&ctx).await.map_err(to_problem)?;
    Ok(Json(result))
}

// ================================================================
// Models
// ================================================================

pub(crate) async fn get_models_summary(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<TradingDashboardService>>,
) -> ApiResult<JsonBody<ModelsSummaryDto>> {
    let result = svc.get_models_summary(&ctx).await.map_err(to_problem)?;
    Ok(Json(result))
}

pub(crate) async fn get_training_history(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<TradingDashboardService>>,
    Query(query): Query<LimitQuery>,
) -> ApiResult<JsonBody<TrainingHistoryListDto>> {
    let result = svc.get_training_history(&ctx, query.limit).await.map_err(to_problem)?;
    Ok(Json(result))
}

pub(crate) async fn get_rejection_reasons(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<TradingDashboardService>>,
) -> ApiResult<JsonBody<RejectionReasonsListDto>> {
    let result = svc.get_rejection_reasons(&ctx).await.map_err(to_problem)?;
    Ok(Json(result))
}

pub(crate) async fn get_symbol_model_status(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<TradingDashboardService>>,
) -> ApiResult<JsonBody<SymbolModelStatusListDto>> {
    let result = svc.get_symbol_model_status(&ctx).await.map_err(to_problem)?;
    Ok(Json(result))
}

pub(crate) async fn get_training_timeline(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<TradingDashboardService>>,
) -> ApiResult<JsonBody<TrainingTimelineListDto>> {
    let result = svc.get_training_timeline(&ctx).await.map_err(to_problem)?;
    Ok(Json(result))
}

pub(crate) async fn get_performance_distribution(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<TradingDashboardService>>,
) -> ApiResult<JsonBody<PerformanceDistributionDto>> {
    let result = svc.get_performance_distribution(&ctx).await.map_err(to_problem)?;
    Ok(Json(result))
}

pub(crate) async fn get_calibration_metrics(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<TradingDashboardService>>,
) -> ApiResult<JsonBody<CalibrationMetricsSummaryDto>> {
    let result = svc.get_calibration_metrics(&ctx).await.map_err(to_problem)?;
    Ok(Json(result))
}

pub(crate) async fn get_regime_analysis(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<TradingDashboardService>>,
) -> ApiResult<JsonBody<RegimeAnalysisListDto>> {
    let result = svc.get_regime_analysis(&ctx).await.map_err(to_problem)?;
    Ok(Json(result))
}

pub(crate) async fn get_model_age_impact(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<TradingDashboardService>>,
) -> ApiResult<JsonBody<ModelAgeImpactListDto>> {
    let result = svc.get_model_age_impact(&ctx).await.map_err(to_problem)?;
    Ok(Json(result))
}

// ================================================================
// Predictions
// ================================================================

pub(crate) async fn get_predictions(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<TradingDashboardService>>,
    Query(query): Query<LimitQuery>,
) -> ApiResult<JsonBody<PredictionsListDto>> {
    let result = svc.get_predictions(&ctx, query.limit).await.map_err(to_problem)?;
    Ok(Json(result))
}

// ================================================================
// Agents
// ================================================================

pub(crate) async fn get_agents_overview(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<TradingDashboardService>>,
) -> ApiResult<JsonBody<AgentOverviewListDto>> {
    let result = svc.get_agents_overview(&ctx).await.map_err(to_problem)?;
    Ok(Json(result))
}

pub(crate) async fn get_agent_decisions_recent(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<TradingDashboardService>>,
    Query(query): Query<LimitQuery>,
) -> ApiResult<JsonBody<AgentDecisionsListDto>> {
    let result = svc.get_agent_decisions_recent(&ctx, query.limit).await.map_err(to_problem)?;
    Ok(Json(result))
}

pub(crate) async fn get_agent_decisions_timeline(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<TradingDashboardService>>,
) -> ApiResult<JsonBody<AgentDecisionTimelineListDto>> {
    let result = svc.get_agent_decisions_timeline(&ctx).await.map_err(to_problem)?;
    Ok(Json(result))
}

pub(crate) async fn get_sentiment_trends(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<TradingDashboardService>>,
) -> ApiResult<JsonBody<SentimentTrendsListDto>> {
    let result = svc.get_sentiment_trends(&ctx).await.map_err(to_problem)?;
    Ok(Json(result))
}

pub(crate) async fn get_agent_performance_metrics(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<TradingDashboardService>>,
) -> ApiResult<JsonBody<AgentPerformanceListDto>> {
    let result = svc.get_agent_performance_metrics(&ctx).await.map_err(to_problem)?;
    Ok(Json(result))
}

pub(crate) async fn get_llm_usage(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<TradingDashboardService>>,
) -> ApiResult<JsonBody<LlmUsageDto>> {
    let result = svc.get_llm_usage(&ctx).await.map_err(to_problem)?;
    Ok(Json(result))
}

pub(crate) async fn get_agent_config(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<TradingDashboardService>>,
) -> ApiResult<JsonBody<AgentConfigListDto>> {
    let result = svc.get_agent_config(&ctx).await.map_err(to_problem)?;
    Ok(Json(result))
}

// ================================================================
// Agent Impact
// ================================================================

pub(crate) async fn get_agent_buy_impact(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<TradingDashboardService>>,
) -> ApiResult<JsonBody<AgentTradeImpactListDto>> {
    let result = svc.get_agent_trade_impacts(&ctx, Some("buy")).await.map_err(to_problem)?;
    Ok(Json(result))
}

pub(crate) async fn get_agent_sell_impact(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<TradingDashboardService>>,
) -> ApiResult<JsonBody<AgentTradeImpactListDto>> {
    let result = svc.get_agent_trade_impacts(&ctx, Some("sell")).await.map_err(to_problem)?;
    Ok(Json(result))
}

pub(crate) async fn get_agent_effectiveness(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<TradingDashboardService>>,
) -> ApiResult<JsonBody<AgentEffectivenessDto>> {
    let result = svc.get_agent_effectiveness(&ctx).await.map_err(to_problem)?;
    Ok(Json(result))
}

// ================================================================
// Performance
// ================================================================

pub(crate) async fn get_performance_overview(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<TradingDashboardService>>,
) -> ApiResult<JsonBody<PerformanceOverviewDto>> {
    let result = svc.get_performance_overview(&ctx).await.map_err(to_problem)?;
    Ok(Json(result))
}

pub(crate) async fn get_strategy_comparison(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<TradingDashboardService>>,
) -> ApiResult<JsonBody<StrategyComparisonListDto>> {
    let result = svc.get_strategy_comparison(&ctx).await.map_err(to_problem)?;
    Ok(Json(result))
}

pub(crate) async fn get_performance_alerts(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<TradingDashboardService>>,
) -> ApiResult<JsonBody<PerformanceAlertsListDto>> {
    let result = svc.get_performance_alerts(&ctx).await.map_err(to_problem)?;
    Ok(Json(result))
}

// ================================================================
// DB Tables
// ================================================================

pub(crate) async fn get_db_tables(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<TradingDashboardService>>,
) -> ApiResult<JsonBody<DbTablesListDto>> {
    let result = svc.get_db_tables(&ctx).await.map_err(to_problem)?;
    Ok(Json(result))
}
