use std::sync::Arc;

use axum::Router;
use modkit::api::OpenApiRegistry;
use modkit::api::operation_builder::{AuthReqAction, AuthReqResource, LicenseFeature, OperationBuilder};
use tracing::info;

use super::dto;
use super::handlers;
use crate::domain::service::AgentAnalyticsService;

// ── Local auth enums ─────────

pub(crate) enum Resource { Agents }

impl AsRef<str> for Resource {
    fn as_ref(&self) -> &str {
        match self { Self::Agents => "agents" }
    }
}

impl AuthReqResource for Resource {}

pub(crate) enum Action { Read }

impl AsRef<str> for Action {
    fn as_ref(&self) -> &str {
        match self { Self::Read => "read" }
    }
}

impl AuthReqAction for Action {}

pub(crate) struct License;

impl AsRef<str> for License {
    fn as_ref(&self) -> &str { "" }
}

impl LicenseFeature for License {}

pub fn register_routes(
    mut router: Router,
    openapi: &dyn OpenApiRegistry,
    service: Arc<AgentAnalyticsService>,
) -> Router {
    info!("Registering agent-analytics REST routes");

    // GET /agent-analytics/v1/overview
    router = OperationBuilder::get("/agent-analytics/v1/overview")
        .operation_id("agent_analytics.get_overview")
        .summary("Get agent overview")
        .description("Get overall agent status and summary statistics")
        .tag("agent-analytics")
        .require_auth(&Resource::Agents, &Action::Read)
        .require_license_features::<License>([])
        .handler(handlers::get_overview)
        .json_response_with_schema::<dto::AgentOverviewResponse>(openapi, http::StatusCode::OK, "Agent overview")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    // GET /agent-analytics/v1/decisions/recent
    router = OperationBuilder::get("/agent-analytics/v1/decisions/recent")
        .operation_id("agent_analytics.get_recent_decisions")
        .summary("Get recent decisions")
        .description("Get recent agent decisions with full details")
        .tag("agent-analytics")
        .require_auth(&Resource::Agents, &Action::Read)
        .require_license_features::<License>([])
        .query_param("agent_type", false, "Filter by agent type")
        .query_param("limit", false, "Number of decisions (default: 50, max: 200)")
        .query_param("hours", false, "Time window in hours (default: 24)")
        .handler(handlers::get_recent_decisions)
        .json_response_with_schema::<dto::RecentDecisionsResponse>(openapi, http::StatusCode::OK, "Recent decisions")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    // GET /agent-analytics/v1/decisions/timeline
    router = OperationBuilder::get("/agent-analytics/v1/decisions/timeline")
        .operation_id("agent_analytics.get_decisions_timeline")
        .summary("Get decisions timeline")
        .description("Get decision timeline data aggregated by hour")
        .tag("agent-analytics")
        .require_auth(&Resource::Agents, &Action::Read)
        .require_license_features::<License>([])
        .query_param("days", false, "Number of days (default: 7, max: 90)")
        .query_param("agent_type", false, "Filter by agent type")
        .handler(handlers::get_decisions_timeline)
        .json_response_with_schema::<dto::DecisionsTimelineResponse>(openapi, http::StatusCode::OK, "Decisions timeline")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    // GET /agent-analytics/v1/sentiment/trends
    router = OperationBuilder::get("/agent-analytics/v1/sentiment/trends")
        .operation_id("agent_analytics.get_sentiment_trends")
        .summary("Get sentiment trends")
        .description("Get sentiment analysis trends over time")
        .tag("agent-analytics")
        .require_auth(&Resource::Agents, &Action::Read)
        .require_license_features::<License>([])
        .query_param("symbol", false, "Filter by symbol")
        .query_param("hours", false, "Time window in hours (default: 24)")
        .handler(handlers::get_sentiment_trends)
        .json_response_with_schema::<dto::SentimentTrendsResponse>(openapi, http::StatusCode::OK, "Sentiment trends")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    // GET /agent-analytics/v1/performance/metrics
    router = OperationBuilder::get("/agent-analytics/v1/performance/metrics")
        .operation_id("agent_analytics.get_performance_metrics")
        .summary("Get performance metrics")
        .description("Get agent performance metrics over time")
        .tag("agent-analytics")
        .require_auth(&Resource::Agents, &Action::Read)
        .require_license_features::<License>([])
        .query_param("days", false, "Number of days (default: 7, max: 90)")
        .query_param("agent_type", false, "Filter by agent type")
        .handler(handlers::get_performance_metrics)
        .json_response_with_schema::<dto::PerformanceMetricsResponse>(openapi, http::StatusCode::OK, "Performance metrics")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    // GET /agent-analytics/v1/llm/usage
    router = OperationBuilder::get("/agent-analytics/v1/llm/usage")
        .operation_id("agent_analytics.get_llm_usage")
        .summary("Get LLM usage")
        .description("Get LLM usage and cost tracking data")
        .tag("agent-analytics")
        .require_auth(&Resource::Agents, &Action::Read)
        .require_license_features::<License>([])
        .query_param("days", false, "Number of days (default: 7, max: 90)")
        .query_param("agent_type", false, "Filter by agent type")
        .handler(handlers::get_llm_usage)
        .json_response_with_schema::<dto::LlmUsageResponse>(openapi, http::StatusCode::OK, "LLM usage")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    // GET /agent-analytics/v1/config
    router = OperationBuilder::get("/agent-analytics/v1/config")
        .operation_id("agent_analytics.get_agent_config")
        .summary("Get agent config")
        .description("Get agent configuration settings")
        .tag("agent-analytics")
        .require_auth(&Resource::Agents, &Action::Read)
        .require_license_features::<License>([])
        .handler(handlers::get_agent_config)
        .json_response_with_schema::<dto::AgentConfigResponse>(openapi, http::StatusCode::OK, "Agent config")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    // GET /agent-analytics/v1/strategy-research/experiments
    router = OperationBuilder::get("/agent-analytics/v1/strategy-research/experiments")
        .operation_id("agent_analytics.get_strategy_experiments")
        .summary("Get strategy experiments")
        .description("Get strategy research experiments")
        .tag("agent-analytics")
        .require_auth(&Resource::Agents, &Action::Read)
        .require_license_features::<License>([])
        .query_param("status", false, "Filter by status")
        .query_param("limit", false, "Number of experiments (default: 50, max: 200)")
        .handler(handlers::get_strategy_experiments)
        .json_response_with_schema::<dto::StrategyExperimentsResponse>(openapi, http::StatusCode::OK, "Strategy experiments")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    // GET /agent-analytics/v1/strategy-research/leaderboard
    router = OperationBuilder::get("/agent-analytics/v1/strategy-research/leaderboard")
        .operation_id("agent_analytics.get_strategy_leaderboard")
        .summary("Get strategy leaderboard")
        .description("Get strategy performance leaderboard")
        .tag("agent-analytics")
        .require_auth(&Resource::Agents, &Action::Read)
        .require_license_features::<License>([])
        .query_param("limit", false, "Number of entries (default: 20, max: 100)")
        .handler(handlers::get_strategy_leaderboard)
        .json_response_with_schema::<dto::StrategyLeaderboardResponse>(openapi, http::StatusCode::OK, "Strategy leaderboard")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    // GET /agent-analytics/v1/strategy-research/indicators
    router = OperationBuilder::get("/agent-analytics/v1/strategy-research/indicators")
        .operation_id("agent_analytics.get_indicator_analysis")
        .summary("Get indicator analysis")
        .description("Get indicator analysis and importance scores")
        .tag("agent-analytics")
        .require_auth(&Resource::Agents, &Action::Read)
        .require_license_features::<License>([])
        .query_param("pair", false, "Filter by pair")
        .query_param("limit", false, "Number of entries (default: 50, max: 200)")
        .handler(handlers::get_indicator_analysis)
        .json_response_with_schema::<dto::IndicatorAnalysisResponse>(openapi, http::StatusCode::OK, "Indicator analysis")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    info!("Agent-analytics REST routes registered successfully");

    router.layer(axum::Extension(service))
}
