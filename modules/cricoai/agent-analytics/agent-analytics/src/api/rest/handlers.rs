use axum::Extension;
use axum::extract::Query;
use modkit::api::prelude::*;
use modkit_security::SecurityContext;
use serde::Deserialize;
use std::sync::Arc;

use crate::api::rest::dto::*;
use crate::domain::service::AgentAnalyticsService;

#[derive(Debug, Deserialize)]
pub struct DecisionsQuery {
    pub agent_type: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default = "default_hours")]
    pub hours: i64,
}

#[derive(Debug, Deserialize)]
pub struct TimelineQuery {
    #[serde(default = "default_days")]
    pub days: i64,
    pub agent_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SentimentQuery {
    pub symbol: Option<String>,
    #[serde(default = "default_hours")]
    pub hours: i64,
}

#[derive(Debug, Deserialize)]
pub struct PerformanceQuery {
    #[serde(default = "default_days")]
    pub days: i64,
    pub agent_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ExperimentsQuery {
    pub status: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

#[derive(Debug, Deserialize)]
pub struct LimitQuery {
    #[serde(default = "default_leaderboard_limit")]
    pub limit: i64,
}

#[derive(Debug, Deserialize)]
pub struct IndicatorQuery {
    pub pair: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 { 50 }
fn default_leaderboard_limit() -> i64 { 20 }
fn default_hours() -> i64 { 24 }
fn default_days() -> i64 { 7 }

pub(crate) async fn get_overview(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<AgentAnalyticsService>>,
) -> ApiResult<JsonBody<AgentOverviewResponse>> {
    let resp = svc.get_overview(&ctx).await?;
    Ok(Json(resp))
}

pub(crate) async fn get_recent_decisions(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<AgentAnalyticsService>>,
    Query(q): Query<DecisionsQuery>,
) -> ApiResult<JsonBody<RecentDecisionsResponse>> {
    let limit = q.limit.min(200).max(1);
    let hours = q.hours.min(720).max(1);
    let resp = svc.get_recent_decisions(&ctx, q.agent_type.as_deref(), limit, hours).await?;
    Ok(Json(resp))
}

pub(crate) async fn get_decisions_timeline(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<AgentAnalyticsService>>,
    Query(q): Query<TimelineQuery>,
) -> ApiResult<JsonBody<DecisionsTimelineResponse>> {
    let days = q.days.min(90).max(1);
    let resp = svc.get_decisions_timeline(&ctx, days, q.agent_type.as_deref()).await?;
    Ok(Json(resp))
}

pub(crate) async fn get_sentiment_trends(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<AgentAnalyticsService>>,
    Query(q): Query<SentimentQuery>,
) -> ApiResult<JsonBody<SentimentTrendsResponse>> {
    let hours = q.hours.min(720).max(1);
    let resp = svc.get_sentiment_trends(&ctx, q.symbol.as_deref(), hours).await?;
    Ok(Json(resp))
}

pub(crate) async fn get_performance_metrics(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<AgentAnalyticsService>>,
    Query(q): Query<PerformanceQuery>,
) -> ApiResult<JsonBody<PerformanceMetricsResponse>> {
    let days = q.days.min(90).max(1);
    let resp = svc.get_performance_metrics(&ctx, days, q.agent_type.as_deref()).await?;
    Ok(Json(resp))
}

pub(crate) async fn get_llm_usage(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<AgentAnalyticsService>>,
    Query(q): Query<PerformanceQuery>,
) -> ApiResult<JsonBody<LlmUsageResponse>> {
    let days = q.days.min(90).max(1);
    let resp = svc.get_llm_usage(&ctx, days, q.agent_type.as_deref()).await?;
    Ok(Json(resp))
}

pub(crate) async fn get_agent_config(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<AgentAnalyticsService>>,
) -> ApiResult<JsonBody<AgentConfigResponse>> {
    let resp = svc.get_agent_config(&ctx).await?;
    Ok(Json(resp))
}

pub(crate) async fn get_strategy_experiments(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<AgentAnalyticsService>>,
    Query(q): Query<ExperimentsQuery>,
) -> ApiResult<JsonBody<StrategyExperimentsResponse>> {
    let limit = q.limit.min(200).max(1);
    let resp = svc.get_strategy_experiments(&ctx, q.status.as_deref(), limit).await?;
    Ok(Json(resp))
}

pub(crate) async fn get_strategy_leaderboard(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<AgentAnalyticsService>>,
    Query(q): Query<LimitQuery>,
) -> ApiResult<JsonBody<StrategyLeaderboardResponse>> {
    let limit = q.limit.min(100).max(1);
    let resp = svc.get_strategy_leaderboard(&ctx, limit).await?;
    Ok(Json(resp))
}

pub(crate) async fn get_indicator_analysis(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<AgentAnalyticsService>>,
    Query(q): Query<IndicatorQuery>,
) -> ApiResult<JsonBody<IndicatorAnalysisResponse>> {
    let limit = q.limit.min(200).max(1);
    let resp = svc.get_indicator_analysis(&ctx, q.pair.as_deref(), limit).await?;
    Ok(Json(resp))
}
