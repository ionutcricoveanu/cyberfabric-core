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

    // Inject the service as Extension for all routes
    router.layer(axum::Extension(service))
}
