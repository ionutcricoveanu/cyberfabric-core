use std::sync::Arc;

use axum::Router;
use modkit::api::OpenApiRegistry;
use modkit::api::operation_builder::{AuthReqAction, AuthReqResource, LicenseFeature, OperationBuilder};
use tracing::info;

use super::dto;
use super::handlers;
use crate::domain::service::PerformanceMonitorService;

// ── Local auth enums ─────────

pub(crate) enum Resource { Performance }

impl AsRef<str> for Resource {
    fn as_ref(&self) -> &str {
        match self { Self::Performance => "performance" }
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
    service: Arc<PerformanceMonitorService>,
) -> Router {
    info!("Registering performance-monitor REST routes");

    // GET /performance-monitor/v1/overview
    router = OperationBuilder::get("/performance-monitor/v1/overview")
        .operation_id("performance_monitor.get_overview")
        .summary("Get trading performance overview")
        .description("Returns trading performance metrics including win rate, profit factor, ML/signal breakdown")
        .tag("performance-monitor")
        .require_auth(&Resource::Performance, &Action::Read)
        .require_license_features::<License>([])
        .query_param("hours", false, "Time period in hours (default: 24)")
        .query_param("env", false, "Environment: production or testnet")
        .handler(handlers::get_overview)
        .json_response_with_schema::<dto::PerformanceOverviewResponse>(openapi, http::StatusCode::OK, "Performance overview")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    // GET /performance-monitor/v1/strategy-comparison
    router = OperationBuilder::get("/performance-monitor/v1/strategy-comparison")
        .operation_id("performance_monitor.get_strategy_comparison")
        .summary("Compare ML vs signal strategies")
        .description("Compare performance between ML-based and signal-based trading strategies")
        .tag("performance-monitor")
        .require_auth(&Resource::Performance, &Action::Read)
        .require_license_features::<License>([])
        .query_param("hours", false, "Time period in hours (default: 24)")
        .query_param("env", false, "Environment: production or testnet")
        .handler(handlers::get_strategy_comparison)
        .json_response_with_schema::<dto::StrategyComparisonResponse>(openapi, http::StatusCode::OK, "Strategy comparison")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    // GET /performance-monitor/v1/alerts
    router = OperationBuilder::get("/performance-monitor/v1/alerts")
        .operation_id("performance_monitor.get_alerts")
        .summary("Get performance alerts")
        .description("Returns performance alerts and warnings based on recent trading activity")
        .tag("performance-monitor")
        .require_auth(&Resource::Performance, &Action::Read)
        .require_license_features::<License>([])
        .query_param("env", false, "Environment: production or testnet")
        .handler(handlers::get_alerts)
        .json_response_with_schema::<dto::AlertsResponse>(openapi, http::StatusCode::OK, "Performance alerts")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    info!("Performance-monitor REST routes registered successfully");

    router.layer(axum::Extension(service))
}
