use std::sync::Arc;

use axum::Router;
use modkit::api::OpenApiRegistry;
use modkit::api::operation_builder::{AuthReqAction, AuthReqResource, LicenseFeature, OperationBuilder};

use crate::api::rest::{dto, handlers};
use crate::domain::service::TradingDashboardService;

// Authorization enums for trading-dashboard routes

pub(crate) enum Resource {
    Stats,
}

pub(crate) enum Action {
    Read,
}

impl AsRef<str> for Resource {
    fn as_ref(&self) -> &'static str {
        match self {
            Self::Stats => "stats",
        }
    }
}

impl AuthReqResource for Resource {}

impl AsRef<str> for Action {
    fn as_ref(&self) -> &'static str {
        match self {
            Self::Read => "read",
        }
    }
}

impl AuthReqAction for Action {}

pub(crate) struct License;

impl AsRef<str> for License {
    fn as_ref(&self) -> &'static str {
        ""
    }
}

impl LicenseFeature for License {}

pub fn register_routes(
    mut router: Router,
    openapi: &dyn OpenApiRegistry,
    service: Arc<TradingDashboardService>,
) -> Router {
    // GET /trading-dashboard/v1/stats/summary
    router = OperationBuilder::get("/trading-dashboard/v1/stats/summary")
        .operation_id("trading_dashboard.get_stats_summary")
        .summary("Get trading statistics summary")
        .description("Retrieve aggregated trading statistics including P&L, open orders, and account value")
        .tag("trading-dashboard")
        .require_auth(&Resource::Stats, &Action::Read)
        .require_license_features::<License>([])
        .query_param("time_range", false, "Time range filter (e.g., 24h, 7d, 30d)")
        .handler(handlers::get_stats_summary)
        .json_response_with_schema::<dto::StatsSummaryDto>(
            openapi,
            http::StatusCode::OK,
            "Trading statistics summary",
        )
        .error_400(openapi)
        .error_500(openapi)
        .register(router, openapi);

    // GET /trading-dashboard/v1/stats/pnl-by-day
    router = OperationBuilder::get("/trading-dashboard/v1/stats/pnl-by-day")
        .operation_id("trading_dashboard.get_pnl_by_day")
        .summary("Get daily P&L breakdown")
        .description("Aggregated daily profit and loss from completed trades")
        .tag("trading-dashboard")
        .require_auth(&Resource::Stats, &Action::Read)
        .require_license_features::<License>([])
        .handler(handlers::get_pnl_by_day)
        .json_response_with_schema::<dto::DailyPnlListDto>(
            openapi,
            http::StatusCode::OK,
            "Daily P&L list",
        )
        .error_400(openapi)
        .error_500(openapi)
        .register(router, openapi);

    // GET /trading-dashboard/v1/stats/recent-trades
    router = OperationBuilder::get("/trading-dashboard/v1/stats/recent-trades")
        .operation_id("trading_dashboard.get_recent_trades")
        .summary("Get recent completed trades")
        .description("Most recent completed sell orders with P&L")
        .tag("trading-dashboard")
        .require_auth(&Resource::Stats, &Action::Read)
        .require_license_features::<License>([])
        .query_param("limit", false, "Number of trades to return (default: 50)")
        .handler(handlers::get_recent_trades)
        .json_response_with_schema::<dto::RecentTradesListDto>(
            openapi,
            http::StatusCode::OK,
            "Recent trades list",
        )
        .error_400(openapi)
        .error_500(openapi)
        .register(router, openapi);

    // GET /trading-dashboard/v1/stats/statistics
    router = OperationBuilder::get("/trading-dashboard/v1/stats/statistics")
        .operation_id("trading_dashboard.get_statistics")
        .summary("Get aggregate trading statistics")
        .description("Overall trading metrics: total P&L, trade count, averages, pair evolution")
        .tag("trading-dashboard")
        .require_auth(&Resource::Stats, &Action::Read)
        .require_license_features::<License>([])
        .handler(handlers::get_statistics)
        .json_response_with_schema::<dto::TradingStatisticsDto>(
            openapi,
            http::StatusCode::OK,
            "Aggregate trading statistics",
        )
        .error_400(openapi)
        .error_500(openapi)
        .register(router, openapi);

    // GET /trading-dashboard/v1/stats/top-pairs
    router = OperationBuilder::get("/trading-dashboard/v1/stats/top-pairs")
        .operation_id("trading_dashboard.get_top_pairs")
        .summary("Get top trading pairs by P&L")
        .description("Ranked trading pairs by total profit/loss with win rates")
        .tag("trading-dashboard")
        .require_auth(&Resource::Stats, &Action::Read)
        .require_license_features::<License>([])
        .handler(handlers::get_top_pairs)
        .json_response_with_schema::<dto::TopPairsListDto>(
            openapi,
            http::StatusCode::OK,
            "Top trading pairs",
        )
        .error_400(openapi)
        .error_500(openapi)
        .register(router, openapi);

    // GET /trading-dashboard/v1/stats/total-assets-value
    router = OperationBuilder::get("/trading-dashboard/v1/stats/total-assets-value")
        .operation_id("trading_dashboard.get_total_assets_value")
        .summary("Get total asset value history")
        .description("Hourly asset value snapshots for the last 24 hours")
        .tag("trading-dashboard")
        .require_auth(&Resource::Stats, &Action::Read)
        .require_license_features::<License>([])
        .handler(handlers::get_total_assets_value)
        .json_response_with_schema::<dto::TotalAssetsValueDto>(
            openapi,
            http::StatusCode::OK,
            "Total asset value history",
        )
        .error_400(openapi)
        .error_500(openapi)
        .register(router, openapi);

    // GET /trading-dashboard/v1/orders/open
    router = OperationBuilder::get("/trading-dashboard/v1/orders/open")
        .operation_id("trading_dashboard.get_open_orders")
        .summary("Get open buy orders")
        .description("All currently open buy orders with estimated profit percentages")
        .tag("trading-dashboard")
        .require_auth(&Resource::Stats, &Action::Read)
        .require_license_features::<License>([])
        .handler(handlers::get_open_orders)
        .json_response_with_schema::<dto::OpenOrdersListDto>(
            openapi,
            http::StatusCode::OK,
            "Open buy orders",
        )
        .error_400(openapi)
        .error_500(openapi)
        .register(router, openapi);

    // GET /trading-dashboard/v1/account/balances
    router = OperationBuilder::get("/trading-dashboard/v1/account/balances")
        .operation_id("trading_dashboard.get_account_balances")
        .summary("Get account balances overview")
        .description("Account balance breakdown: USDC, asset value, invested, P&L")
        .tag("trading-dashboard")
        .require_auth(&Resource::Stats, &Action::Read)
        .require_license_features::<License>([])
        .handler(handlers::get_account_balances)
        .json_response_with_schema::<dto::AccountBalancesDto>(
            openapi,
            http::StatusCode::OK,
            "Account balances",
        )
        .error_400(openapi)
        .error_500(openapi)
        .register(router, openapi);

    // ================================================================
    // Trading Orders
    // ================================================================

    // GET /trading-dashboard/v1/orders/pairs-evolution
    router = OperationBuilder::get("/trading-dashboard/v1/orders/pairs-evolution")
        .operation_id("trading_dashboard.get_pairs_evolution")
        .summary("Get pairs price evolution")
        .description("Current price evolution data for all trading pairs")
        .tag("trading-dashboard")
        .require_auth(&Resource::Stats, &Action::Read)
        .require_license_features::<License>([])
        .handler(handlers::get_pairs_evolution)
        .json_response_with_schema::<dto::PairsEvolutionListDto>(openapi, http::StatusCode::OK, "Pairs evolution")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    // GET /trading-dashboard/v1/orders/excluded
    router = OperationBuilder::get("/trading-dashboard/v1/orders/excluded")
        .operation_id("trading_dashboard.get_excluded_pairs")
        .summary("Get excluded trading pairs")
        .description("Pairs excluded from trading")
        .tag("trading-dashboard")
        .require_auth(&Resource::Stats, &Action::Read)
        .require_license_features::<License>([])
        .handler(handlers::get_excluded_pairs)
        .json_response_with_schema::<dto::ExcludedPairsListDto>(openapi, http::StatusCode::OK, "Excluded pairs")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    // ================================================================
    // Models
    // ================================================================

    // GET /trading-dashboard/v1/models/summary
    router = OperationBuilder::get("/trading-dashboard/v1/models/summary")
        .operation_id("trading_dashboard.get_models_summary")
        .summary("Get models training summary")
        .description("Aggregate model training statistics")
        .tag("trading-dashboard")
        .require_auth(&Resource::Stats, &Action::Read)
        .require_license_features::<License>([])
        .handler(handlers::get_models_summary)
        .json_response_with_schema::<dto::ModelsSummaryDto>(openapi, http::StatusCode::OK, "Models summary")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    // GET /trading-dashboard/v1/models/training-history
    router = OperationBuilder::get("/trading-dashboard/v1/models/training-history")
        .operation_id("trading_dashboard.get_training_history")
        .summary("Get model training history")
        .description("Chronological list of model training runs")
        .tag("trading-dashboard")
        .require_auth(&Resource::Stats, &Action::Read)
        .require_license_features::<License>([])
        .query_param("limit", false, "Number of records (default: 50)")
        .handler(handlers::get_training_history)
        .json_response_with_schema::<dto::TrainingHistoryListDto>(openapi, http::StatusCode::OK, "Training history")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    // GET /trading-dashboard/v1/models/rejection-reasons
    router = OperationBuilder::get("/trading-dashboard/v1/models/rejection-reasons")
        .operation_id("trading_dashboard.get_rejection_reasons")
        .summary("Get model rejection reasons")
        .description("Aggregated reasons why models were rejected")
        .tag("trading-dashboard")
        .require_auth(&Resource::Stats, &Action::Read)
        .require_license_features::<License>([])
        .handler(handlers::get_rejection_reasons)
        .json_response_with_schema::<dto::RejectionReasonsListDto>(openapi, http::StatusCode::OK, "Rejection reasons")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    // GET /trading-dashboard/v1/models/symbol-status
    router = OperationBuilder::get("/trading-dashboard/v1/models/symbol-status")
        .operation_id("trading_dashboard.get_symbol_model_status")
        .summary("Get per-symbol model status")
        .description("Training status per symbol with latest results")
        .tag("trading-dashboard")
        .require_auth(&Resource::Stats, &Action::Read)
        .require_license_features::<License>([])
        .handler(handlers::get_symbol_model_status)
        .json_response_with_schema::<dto::SymbolModelStatusListDto>(openapi, http::StatusCode::OK, "Symbol model status")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    // GET /trading-dashboard/v1/models/training-timeline
    router = OperationBuilder::get("/trading-dashboard/v1/models/training-timeline")
        .operation_id("trading_dashboard.get_training_timeline")
        .summary("Get training timeline")
        .description("Daily training activity over time")
        .tag("trading-dashboard")
        .require_auth(&Resource::Stats, &Action::Read)
        .require_license_features::<License>([])
        .handler(handlers::get_training_timeline)
        .json_response_with_schema::<dto::TrainingTimelineListDto>(openapi, http::StatusCode::OK, "Training timeline")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    // GET /trading-dashboard/v1/models/performance-distribution
    router = OperationBuilder::get("/trading-dashboard/v1/models/performance-distribution")
        .operation_id("trading_dashboard.get_performance_distribution")
        .summary("Get model performance distribution")
        .description("Win rate and Sharpe ratio distribution of promoted models")
        .tag("trading-dashboard")
        .require_auth(&Resource::Stats, &Action::Read)
        .require_license_features::<License>([])
        .handler(handlers::get_performance_distribution)
        .json_response_with_schema::<dto::PerformanceDistributionDto>(openapi, http::StatusCode::OK, "Performance distribution")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    // GET /trading-dashboard/v1/models/calibration-metrics
    router = OperationBuilder::get("/trading-dashboard/v1/models/calibration-metrics")
        .operation_id("trading_dashboard.get_calibration_metrics")
        .summary("Get calibration metrics summary")
        .description("Model calibration accuracy and confidence analysis")
        .tag("trading-dashboard")
        .require_auth(&Resource::Stats, &Action::Read)
        .require_license_features::<License>([])
        .handler(handlers::get_calibration_metrics)
        .json_response_with_schema::<dto::CalibrationMetricsSummaryDto>(openapi, http::StatusCode::OK, "Calibration metrics")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    // GET /trading-dashboard/v1/models/regime-analysis
    router = OperationBuilder::get("/trading-dashboard/v1/models/regime-analysis")
        .operation_id("trading_dashboard.get_regime_analysis")
        .summary("Get regime analysis")
        .description("Model performance broken down by market regime")
        .tag("trading-dashboard")
        .require_auth(&Resource::Stats, &Action::Read)
        .require_license_features::<License>([])
        .handler(handlers::get_regime_analysis)
        .json_response_with_schema::<dto::RegimeAnalysisListDto>(openapi, http::StatusCode::OK, "Regime analysis")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    // GET /trading-dashboard/v1/models/model-age-impact
    router = OperationBuilder::get("/trading-dashboard/v1/models/model-age-impact")
        .operation_id("trading_dashboard.get_model_age_impact")
        .summary("Get model age impact")
        .description("How model age affects prediction accuracy")
        .tag("trading-dashboard")
        .require_auth(&Resource::Stats, &Action::Read)
        .require_license_features::<License>([])
        .handler(handlers::get_model_age_impact)
        .json_response_with_schema::<dto::ModelAgeImpactListDto>(openapi, http::StatusCode::OK, "Model age impact")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    // ================================================================
    // Predictions
    // ================================================================

    // GET /trading-dashboard/v1/predictions
    router = OperationBuilder::get("/trading-dashboard/v1/predictions")
        .operation_id("trading_dashboard.get_predictions")
        .summary("Get prediction history")
        .description("Recent model predictions with outcomes")
        .tag("trading-dashboard")
        .require_auth(&Resource::Stats, &Action::Read)
        .require_license_features::<License>([])
        .query_param("limit", false, "Number of predictions (default: 50)")
        .handler(handlers::get_predictions)
        .json_response_with_schema::<dto::PredictionsListDto>(openapi, http::StatusCode::OK, "Predictions list")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    // ================================================================
    // Agents
    // ================================================================

    // GET /trading-dashboard/v1/agents/overview
    router = OperationBuilder::get("/trading-dashboard/v1/agents/overview")
        .operation_id("trading_dashboard.get_agents_overview")
        .summary("Get agents overview")
        .description("Summary of all agent types with decision counts and costs")
        .tag("trading-dashboard")
        .require_auth(&Resource::Stats, &Action::Read)
        .require_license_features::<License>([])
        .handler(handlers::get_agents_overview)
        .json_response_with_schema::<dto::AgentOverviewListDto>(openapi, http::StatusCode::OK, "Agents overview")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    // GET /trading-dashboard/v1/agents/decisions/recent
    router = OperationBuilder::get("/trading-dashboard/v1/agents/decisions/recent")
        .operation_id("trading_dashboard.get_agent_decisions_recent")
        .summary("Get recent agent decisions")
        .description("Most recent agent decisions with confidence and status")
        .tag("trading-dashboard")
        .require_auth(&Resource::Stats, &Action::Read)
        .require_license_features::<License>([])
        .query_param("limit", false, "Number of decisions (default: 50)")
        .handler(handlers::get_agent_decisions_recent)
        .json_response_with_schema::<dto::AgentDecisionsListDto>(openapi, http::StatusCode::OK, "Recent decisions")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    // GET /trading-dashboard/v1/agents/decisions/timeline
    router = OperationBuilder::get("/trading-dashboard/v1/agents/decisions/timeline")
        .operation_id("trading_dashboard.get_agent_decisions_timeline")
        .summary("Get agent decisions timeline")
        .description("Daily agent decision activity over time")
        .tag("trading-dashboard")
        .require_auth(&Resource::Stats, &Action::Read)
        .require_license_features::<License>([])
        .handler(handlers::get_agent_decisions_timeline)
        .json_response_with_schema::<dto::AgentDecisionTimelineListDto>(openapi, http::StatusCode::OK, "Decisions timeline")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    // GET /trading-dashboard/v1/agents/sentiment/trends
    router = OperationBuilder::get("/trading-dashboard/v1/agents/sentiment/trends")
        .operation_id("trading_dashboard.get_sentiment_trends")
        .summary("Get sentiment trends")
        .description("Daily average sentiment scores over time")
        .tag("trading-dashboard")
        .require_auth(&Resource::Stats, &Action::Read)
        .require_license_features::<License>([])
        .handler(handlers::get_sentiment_trends)
        .json_response_with_schema::<dto::SentimentTrendsListDto>(openapi, http::StatusCode::OK, "Sentiment trends")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    // GET /trading-dashboard/v1/agents/performance/metrics
    router = OperationBuilder::get("/trading-dashboard/v1/agents/performance/metrics")
        .operation_id("trading_dashboard.get_agent_performance_metrics")
        .summary("Get agent performance metrics")
        .description("Daily performance metrics per agent type")
        .tag("trading-dashboard")
        .require_auth(&Resource::Stats, &Action::Read)
        .require_license_features::<License>([])
        .handler(handlers::get_agent_performance_metrics)
        .json_response_with_schema::<dto::AgentPerformanceListDto>(openapi, http::StatusCode::OK, "Agent performance")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    // GET /trading-dashboard/v1/agents/llm/usage
    router = OperationBuilder::get("/trading-dashboard/v1/agents/llm/usage")
        .operation_id("trading_dashboard.get_llm_usage")
        .summary("Get LLM usage statistics")
        .description("Token usage, costs, and processing times by model")
        .tag("trading-dashboard")
        .require_auth(&Resource::Stats, &Action::Read)
        .require_license_features::<License>([])
        .handler(handlers::get_llm_usage)
        .json_response_with_schema::<dto::LlmUsageDto>(openapi, http::StatusCode::OK, "LLM usage")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    // GET /trading-dashboard/v1/agents/config
    router = OperationBuilder::get("/trading-dashboard/v1/agents/config")
        .operation_id("trading_dashboard.get_agent_config")
        .summary("Get agent configurations")
        .description("Current configuration for all agent types")
        .tag("trading-dashboard")
        .require_auth(&Resource::Stats, &Action::Read)
        .require_license_features::<License>([])
        .handler(handlers::get_agent_config)
        .json_response_with_schema::<dto::AgentConfigListDto>(openapi, http::StatusCode::OK, "Agent config")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    // ================================================================
    // Agent Impact
    // ================================================================

    // GET /trading-dashboard/v1/agents/impact/buy-impact
    router = OperationBuilder::get("/trading-dashboard/v1/agents/impact/buy-impact")
        .operation_id("trading_dashboard.get_agent_buy_impact")
        .summary("Get agent buy impact")
        .description("Trade impact analysis for agent-influenced buys")
        .tag("trading-dashboard")
        .require_auth(&Resource::Stats, &Action::Read)
        .require_license_features::<License>([])
        .handler(handlers::get_agent_buy_impact)
        .json_response_with_schema::<dto::AgentTradeImpactListDto>(openapi, http::StatusCode::OK, "Buy impact")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    // GET /trading-dashboard/v1/agents/impact/sell-impact
    router = OperationBuilder::get("/trading-dashboard/v1/agents/impact/sell-impact")
        .operation_id("trading_dashboard.get_agent_sell_impact")
        .summary("Get agent sell impact")
        .description("Trade impact analysis for agent-influenced sells")
        .tag("trading-dashboard")
        .require_auth(&Resource::Stats, &Action::Read)
        .require_license_features::<License>([])
        .handler(handlers::get_agent_sell_impact)
        .json_response_with_schema::<dto::AgentTradeImpactListDto>(openapi, http::StatusCode::OK, "Sell impact")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    // GET /trading-dashboard/v1/agents/impact/effectiveness
    router = OperationBuilder::get("/trading-dashboard/v1/agents/impact/effectiveness")
        .operation_id("trading_dashboard.get_agent_effectiveness")
        .summary("Get agent effectiveness")
        .description("Overall agent effectiveness compared to counterfactual")
        .tag("trading-dashboard")
        .require_auth(&Resource::Stats, &Action::Read)
        .require_license_features::<License>([])
        .handler(handlers::get_agent_effectiveness)
        .json_response_with_schema::<dto::AgentEffectivenessDto>(openapi, http::StatusCode::OK, "Agent effectiveness")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    // ================================================================
    // Performance
    // ================================================================

    // GET /trading-dashboard/v1/performance/overview
    router = OperationBuilder::get("/trading-dashboard/v1/performance/overview")
        .operation_id("trading_dashboard.get_performance_overview")
        .summary("Get performance overview")
        .description("Overall trading performance with win rate, drawdown, streaks")
        .tag("trading-dashboard")
        .require_auth(&Resource::Stats, &Action::Read)
        .require_license_features::<License>([])
        .handler(handlers::get_performance_overview)
        .json_response_with_schema::<dto::PerformanceOverviewDto>(openapi, http::StatusCode::OK, "Performance overview")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    // GET /trading-dashboard/v1/performance/strategy-comparison
    router = OperationBuilder::get("/trading-dashboard/v1/performance/strategy-comparison")
        .operation_id("trading_dashboard.get_strategy_comparison")
        .summary("Get strategy comparison")
        .description("Performance comparison across different buy strategies")
        .tag("trading-dashboard")
        .require_auth(&Resource::Stats, &Action::Read)
        .require_license_features::<License>([])
        .handler(handlers::get_strategy_comparison)
        .json_response_with_schema::<dto::StrategyComparisonListDto>(openapi, http::StatusCode::OK, "Strategy comparison")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    // GET /trading-dashboard/v1/performance/alerts
    router = OperationBuilder::get("/trading-dashboard/v1/performance/alerts")
        .operation_id("trading_dashboard.get_performance_alerts")
        .summary("Get performance alerts")
        .description("Active performance alerts (losing streaks, significant losses)")
        .tag("trading-dashboard")
        .require_auth(&Resource::Stats, &Action::Read)
        .require_license_features::<License>([])
        .handler(handlers::get_performance_alerts)
        .json_response_with_schema::<dto::PerformanceAlertsListDto>(openapi, http::StatusCode::OK, "Performance alerts")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    // ================================================================
    // DB Tables
    // ================================================================

    // GET /trading-dashboard/v1/db/tables
    router = OperationBuilder::get("/trading-dashboard/v1/db/tables")
        .operation_id("trading_dashboard.get_db_tables")
        .summary("Get database tables")
        .description("List all known tables with row counts")
        .tag("trading-dashboard")
        .require_auth(&Resource::Stats, &Action::Read)
        .require_license_features::<License>([])
        .handler(handlers::get_db_tables)
        .json_response_with_schema::<dto::DbTablesListDto>(openapi, http::StatusCode::OK, "DB tables")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    // Inject the service as Extension for all routes
    router.layer(axum::Extension(service))
}
