use axum::Extension;
use axum::extract::Query;
use modkit::api::prelude::*;
use modkit_security::SecurityContext;
use serde::Deserialize;
use std::sync::Arc;
use tracing::info;

use crate::api::rest::dto::{
    AccountBalancesDto, DailyPnlListDto, OpenOrdersListDto, RecentTradesListDto,
    StatsSummaryDto, TopPairsListDto, TotalAssetsValueDto, TradingStatisticsDto,
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
