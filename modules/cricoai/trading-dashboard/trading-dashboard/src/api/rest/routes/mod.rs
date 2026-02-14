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

    // Inject the service as Extension for all routes
    router.layer(axum::Extension(service))
}
