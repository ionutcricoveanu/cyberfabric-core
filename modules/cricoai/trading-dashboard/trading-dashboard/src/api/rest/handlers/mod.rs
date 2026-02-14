use axum::Extension;
use axum::extract::Query;
use modkit::api::prelude::*;
use modkit_security::SecurityContext;
use serde::Deserialize;
use std::sync::Arc;
use tracing::info;

use crate::api::rest::dto::StatsSummaryDto;
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

pub(crate) async fn get_stats_summary(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<TradingDashboardService>>,
    Query(query): Query<StatsQuery>,
) -> ApiResult<JsonBody<StatsSummaryDto>> {
    info!(
        subject_id = %ctx.subject_id(),
        time_range = %query.time_range,
        "GET /trading-dashboard/v1/stats/summary"
    );

    let summary = svc
        .get_stats_summary(&ctx, &query.time_range)
        .await
        .map_err(to_problem)?;
    Ok(Json(StatsSummaryDto::from(summary)))
}
