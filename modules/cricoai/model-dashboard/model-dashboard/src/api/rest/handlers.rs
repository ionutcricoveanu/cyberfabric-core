use axum::Extension;
use axum::extract::Query;
use modkit::api::prelude::*;
use modkit_security::SecurityContext;
use serde::Deserialize;
use std::sync::Arc;

use crate::api::rest::dto::*;
use crate::domain::service::ModelDashboardService;

#[derive(Debug, Deserialize)]
pub struct SummaryQuery {
    #[serde(default = "default_interval")]
    pub interval: String,
    #[serde(default = "default_time_range")]
    pub time_range: String,
}

#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    #[serde(default = "default_interval")]
    pub interval: String,
    #[serde(default = "default_time_range")]
    pub time_range: String,
    pub symbol: Option<String>,
    #[serde(default)]
    pub promoted_only: bool,
    #[serde(default)]
    pub rejected_only: bool,
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

#[derive(Debug, Deserialize)]
pub struct IntervalQuery {
    #[serde(default = "default_interval")]
    pub interval: String,
}

#[derive(Debug, Deserialize)]
pub struct TimeRangeQuery {
    #[serde(default = "default_time_range")]
    pub time_range: String,
}

#[derive(Debug, Deserialize)]
pub struct DistributionQuery {
    #[serde(default = "default_interval")]
    pub interval: String,
    #[serde(default = "default_time_range")]
    pub time_range: String,
    #[serde(default = "default_true")]
    pub promoted_only: bool,
}

fn default_interval() -> String { "all".to_string() }
fn default_time_range() -> String { "7d".to_string() }
fn default_limit() -> i64 { 100 }
fn default_true() -> bool { true }

pub(crate) async fn get_summary(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<ModelDashboardService>>,
    Query(q): Query<SummaryQuery>,
) -> ApiResult<JsonBody<ModelSummaryResponse>> {
    let resp = svc.get_summary(&ctx, &q.interval, &q.time_range).await?;
    Ok(Json(resp))
}

pub(crate) async fn get_training_history(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<ModelDashboardService>>,
    Query(q): Query<HistoryQuery>,
) -> ApiResult<JsonBody<TrainingHistoryResponse>> {
    let limit = q.limit.min(500).max(1);
    let resp = svc.get_training_history(
        &ctx, &q.interval, &q.time_range,
        q.symbol.as_deref(), q.promoted_only, q.rejected_only,
        limit, q.offset,
    ).await?;
    Ok(Json(resp))
}

pub(crate) async fn get_rejection_reasons(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<ModelDashboardService>>,
    Query(q): Query<SummaryQuery>,
) -> ApiResult<JsonBody<RejectionReasonsResponse>> {
    let resp = svc.get_rejection_reasons(&ctx, &q.interval, &q.time_range).await?;
    Ok(Json(resp))
}

pub(crate) async fn get_symbol_status(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<ModelDashboardService>>,
    Query(q): Query<IntervalQuery>,
) -> ApiResult<JsonBody<SymbolStatusResponse>> {
    let resp = svc.get_symbol_status(&ctx, &q.interval).await?;
    Ok(Json(resp))
}

pub(crate) async fn get_training_timeline(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<ModelDashboardService>>,
    Query(q): Query<SummaryQuery>,
) -> ApiResult<JsonBody<TrainingTimelineResponse>> {
    let resp = svc.get_training_timeline(&ctx, &q.interval, &q.time_range).await?;
    Ok(Json(resp))
}

pub(crate) async fn get_performance_distribution(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<ModelDashboardService>>,
    Query(q): Query<DistributionQuery>,
) -> ApiResult<JsonBody<PerformanceDistributionResponse>> {
    let resp = svc.get_performance_distribution(
        &ctx, &q.interval, &q.time_range, q.promoted_only,
    ).await?;
    Ok(Json(resp))
}

pub(crate) async fn get_calibration_metrics(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<ModelDashboardService>>,
    Query(q): Query<TimeRangeQuery>,
) -> ApiResult<JsonBody<CalibrationMetricsResponse>> {
    let resp = svc.get_calibration_metrics(&ctx, &q.time_range).await?;
    Ok(Json(resp))
}

pub(crate) async fn get_regime_analysis(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<ModelDashboardService>>,
    Query(q): Query<TimeRangeQuery>,
) -> ApiResult<JsonBody<RegimeAnalysisResponse>> {
    let resp = svc.get_regime_analysis(&ctx, &q.time_range).await?;
    Ok(Json(resp))
}

pub(crate) async fn get_model_age_impact(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<ModelDashboardService>>,
    Query(q): Query<TimeRangeQuery>,
) -> ApiResult<JsonBody<AgeImpactResponse>> {
    let resp = svc.get_model_age_impact(&ctx, &q.time_range).await?;
    Ok(Json(resp))
}
