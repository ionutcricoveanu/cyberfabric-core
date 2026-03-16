//! REST API route registration for trading-core module

use std::sync::Arc;

use axum::Router;
use modkit::api::OpenApiRegistry;
use modkit::api::operation_builder::{AuthReqAction, AuthReqResource, LicenseFeature, OperationBuilder};

use crate::domain::service::TradingCoreService;
use super::handlers;

pub(crate) enum Resource {
    Trading,
}

pub(crate) enum Action {
    Read,
    Write,
}

impl AsRef<str> for Resource {
    fn as_ref(&self) -> &'static str {
        "trading-core"
    }
}

impl AuthReqResource for Resource {}

impl AsRef<str> for Action {
    fn as_ref(&self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Write => "write",
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

/// Register all REST API routes for the trading-core module
pub fn register_routes(
    mut router: Router,
    openapi: &dyn OpenApiRegistry,
    service: Arc<TradingCoreService>,
) -> Router {
    router = OperationBuilder::get("/trading-core/v1/status")
        .operation_id("trading_core.get_status")
        .summary("Get bot status")
        .description("Retrieve current trading bot status")
        .tag("trading-core")
        .require_auth(&Resource::Trading, &Action::Read)
        .require_license_features::<License>([])
        .handler(handlers::get_status)
        .json_response_with_schema::<handlers::BotStatusResponse>(
            openapi,
            http::StatusCode::OK,
            "Bot status",
        )
        .error_400(openapi)
        .error_500(openapi)
        .register(router, openapi);

    router = OperationBuilder::post("/trading-core/v1/pause")
        .operation_id("trading_core.pause")
        .summary("Pause trading")
        .description("Pause the trading bot")
        .tag("trading-core")
        .require_auth(&Resource::Trading, &Action::Write)
        .require_license_features::<License>([])
        .handler(handlers::pause_trading)
        .json_response_with_schema::<handlers::ActionResponse>(
            openapi,
            http::StatusCode::OK,
            "Pause response",
        )
        .error_400(openapi)
        .error_500(openapi)
        .register(router, openapi);

    router = OperationBuilder::post("/trading-core/v1/resume")
        .operation_id("trading_core.resume")
        .summary("Resume trading")
        .description("Resume the trading bot")
        .tag("trading-core")
        .require_auth(&Resource::Trading, &Action::Write)
        .require_license_features::<License>([])
        .handler(handlers::resume_trading)
        .json_response_with_schema::<handlers::ActionResponse>(
            openapi,
            http::StatusCode::OK,
            "Resume response",
        )
        .error_400(openapi)
        .error_500(openapi)
        .register(router, openapi);

    router = OperationBuilder::get("/trading-core/v1/positions")
        .operation_id("trading_core.get_positions")
        .summary("Get open positions")
        .description("Retrieve all open positions")
        .tag("trading-core")
        .require_auth(&Resource::Trading, &Action::Read)
        .require_license_features::<License>([])
        .handler(handlers::get_positions)
        .json_response_with_schema::<handlers::PositionsResponse>(
            openapi,
            http::StatusCode::OK,
            "Positions list",
        )
        .error_400(openapi)
        .error_500(openapi)
        .register(router, openapi);

    router = OperationBuilder::get("/trading-core/v1/positions/{symbol}")
        .operation_id("trading_core.get_position")
        .summary("Get position")
        .description("Retrieve position details for a symbol")
        .tag("trading-core")
        .require_auth(&Resource::Trading, &Action::Read)
        .require_license_features::<License>([])
        .handler(handlers::get_position)
        .json_response_with_schema::<handlers::PositionResponse>(
            openapi,
            http::StatusCode::OK,
            "Position details",
        )
        .error_400(openapi)
        .error_500(openapi)
        .register(router, openapi);

    router = OperationBuilder::get("/trading-core/v1/order-history")
        .operation_id("trading_core.get_order_history")
        .summary("Get order history")
        .description("Retrieve historical orders")
        .tag("trading-core")
        .require_auth(&Resource::Trading, &Action::Read)
        .require_license_features::<License>([])
        .handler(handlers::get_order_history)
        .json_response_with_schema::<handlers::OrderHistoryResponse>(
            openapi,
            http::StatusCode::OK,
            "Order history",
        )
        .error_400(openapi)
        .error_500(openapi)
        .register(router, openapi);

    router = OperationBuilder::get("/trading-core/v1/summary")
        .operation_id("trading_core.get_summary")
        .summary("Get trading summary")
        .description("Retrieve summary metrics")
        .tag("trading-core")
        .require_auth(&Resource::Trading, &Action::Read)
        .require_license_features::<License>([])
        .handler(handlers::get_trading_summary)
        .json_response_with_schema::<handlers::TradingSummaryResponse>(
            openapi,
            http::StatusCode::OK,
            "Trading summary",
        )
        .error_400(openapi)
        .error_500(openapi)
        .register(router, openapi);

    router = OperationBuilder::get("/trading-core/v1/health")
        .operation_id("trading_core.health")
        .summary("Health check")
        .description("Check trading-core health")
        .tag("trading-core")
        .require_auth(&Resource::Trading, &Action::Read)
        .require_license_features::<License>([])
        .handler(handlers::health)
        .json_response_with_schema::<handlers::HealthResponse>(
            openapi,
            http::StatusCode::OK,
            "Health status",
        )
        .error_500(openapi)
        .register(router, openapi);

    router.layer(axum::Extension(service))
}
