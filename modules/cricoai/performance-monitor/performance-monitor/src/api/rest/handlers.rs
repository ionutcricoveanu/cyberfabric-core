use axum::Extension;
use axum::extract::Query;
use modkit::api::prelude::*;
use modkit_security::SecurityContext;
use serde::Deserialize;
use std::sync::Arc;

use crate::api::rest::dto::*;
use crate::domain::service::PerformanceMonitorService;

#[derive(Debug, Deserialize)]
pub struct OverviewQuery {
    #[serde(default = "default_hours")]
    pub hours: i64,
    #[serde(default = "default_env")]
    pub env: String,
}

#[derive(Debug, Deserialize)]
pub struct EnvQuery {
    #[serde(default = "default_env")]
    pub env: String,
}

fn default_hours() -> i64 { 24 }
fn default_env() -> String { "production".to_string() }

pub(crate) async fn get_overview(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<PerformanceMonitorService>>,
    Query(q): Query<OverviewQuery>,
) -> ApiResult<JsonBody<PerformanceOverviewResponse>> {
    let resp = svc.get_overview(&ctx, &q.env, q.hours).await?;
    Ok(Json(resp))
}

pub(crate) async fn get_strategy_comparison(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<PerformanceMonitorService>>,
    Query(q): Query<OverviewQuery>,
) -> ApiResult<JsonBody<StrategyComparisonResponse>> {
    let resp = svc.get_strategy_comparison(&ctx, &q.env, q.hours).await?;
    Ok(Json(resp))
}

pub(crate) async fn get_alerts(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<PerformanceMonitorService>>,
    Query(q): Query<EnvQuery>,
) -> ApiResult<JsonBody<AlertsResponse>> {
    let resp = svc.get_alerts(&ctx, &q.env).await?;
    Ok(Json(resp))
}
