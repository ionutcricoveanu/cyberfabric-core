use std::sync::Arc;

use axum::Router;
use modkit::api::OpenApiRegistry;
use modkit::api::operation_builder::{AuthReqAction, AuthReqResource, LicenseFeature, OperationBuilder};
use tracing::info;

use super::dto;
use super::handlers;
use crate::domain::service::AgentTradeImpactService;

// ── Local auth enums ─────────

pub(crate) enum Resource { AgentImpact }

impl AsRef<str> for Resource {
    fn as_ref(&self) -> &str {
        match self { Self::AgentImpact => "agent-impact" }
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
    service: Arc<AgentTradeImpactService>,
) -> Router {
    info!("Registering agent-trade-impact REST routes");

    // GET /agent-trade-impact/v1/buy-impact
    router = OperationBuilder::get("/agent-trade-impact/v1/buy-impact")
        .operation_id("agent_trade_impact.get_buy_impact")
        .summary("Get buy trade impact")
        .description("Get agent impact on buy decisions grouped by pair")
        .tag("agent-trade-impact")
        .require_auth(&Resource::AgentImpact, &Action::Read)
        .require_license_features::<License>([])
        .query_param("env", false, "Environment: production or testnet")
        .query_param("days", false, "Period in days (default: 7, max: 90)")
        .query_param("pair", false, "Filter by trading pair")
        .handler(handlers::get_buy_impact)
        .json_response_with_schema::<dto::BuyImpactResponse>(openapi, http::StatusCode::OK, "Buy impact metrics")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    // GET /agent-trade-impact/v1/sell-impact
    router = OperationBuilder::get("/agent-trade-impact/v1/sell-impact")
        .operation_id("agent_trade_impact.get_sell_impact")
        .summary("Get sell trade impact")
        .description("Get agent impact on sell decisions grouped by pair")
        .tag("agent-trade-impact")
        .require_auth(&Resource::AgentImpact, &Action::Read)
        .require_license_features::<License>([])
        .query_param("env", false, "Environment: production or testnet")
        .query_param("days", false, "Period in days (default: 7, max: 90)")
        .query_param("pair", false, "Filter by trading pair")
        .handler(handlers::get_sell_impact)
        .json_response_with_schema::<dto::SellImpactResponse>(openapi, http::StatusCode::OK, "Sell impact metrics")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    // GET /agent-trade-impact/v1/effectiveness
    router = OperationBuilder::get("/agent-trade-impact/v1/effectiveness")
        .operation_id("agent_trade_impact.get_effectiveness")
        .summary("Get agent effectiveness")
        .description("Get overall agent effectiveness metrics with action breakdown")
        .tag("agent-trade-impact")
        .require_auth(&Resource::AgentImpact, &Action::Read)
        .require_license_features::<License>([])
        .query_param("env", false, "Environment: production or testnet")
        .query_param("days", false, "Period in days (default: 30, max: 365)")
        .handler(handlers::get_effectiveness)
        .json_response_with_schema::<dto::EffectivenessResponse>(openapi, http::StatusCode::OK, "Agent effectiveness")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    info!("Agent-trade-impact REST routes registered successfully");

    router.layer(axum::Extension(service))
}
