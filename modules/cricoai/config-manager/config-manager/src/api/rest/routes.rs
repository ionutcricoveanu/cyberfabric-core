use std::sync::Arc;

use axum::Router;
use modkit::api::OpenApiRegistry;
use modkit::api::operation_builder::{AuthReqAction, AuthReqResource, LicenseFeature, OperationBuilder};

use super::dto;
use super::handlers;
use crate::domain::service::ConfigManagerService;

pub(crate) enum Resource {
    Config,
}

pub(crate) enum Action {
    Read,
    Write,
}

impl AsRef<str> for Resource {
    fn as_ref(&self) -> &'static str {
        match self {
            Self::Config => "config",
        }
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

pub fn register_routes(
    mut router: Router,
    openapi: &dyn OpenApiRegistry,
    service: Arc<ConfigManagerService>,
) -> Router {
    // ================================================================
    // Config endpoints
    // ================================================================

    // GET /config-manager/v1/config
    router = OperationBuilder::get("/config-manager/v1/config")
        .operation_id("config_manager.get_config")
        .summary("Get structured configuration")
        .description("Returns trading parameters, indicator settings, and system config from YAML")
        .tag("config-manager")
        .require_auth(&Resource::Config, &Action::Read)
        .require_license_features::<License>([])
        .query_param("env", false, "Environment: production, testnet, testnet_scalping")
        .handler(handlers::get_config)
        .json_response_with_schema::<dto::ConfigResponse>(openapi, http::StatusCode::OK, "Structured config")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    // PUT /config-manager/v1/config
    router = OperationBuilder::put("/config-manager/v1/config")
        .operation_id("config_manager.update_config")
        .summary("Update configuration section")
        .description("Update a config section: trade, indicators, intervals, ml, profit, system")
        .tag("config-manager")
        .require_auth(&Resource::Config, &Action::Write)
        .require_license_features::<License>([])
        .query_param("env", false, "Environment: production, testnet, testnet_scalping")
        .json_request::<dto::ConfigUpdateRequest>(openapi, "Config update payload")
        .handler(handlers::update_config)
        .json_response_with_schema::<dto::ConfigUpdateResponse>(openapi, http::StatusCode::OK, "Update result")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    // GET /config-manager/v1/config/raw
    router = OperationBuilder::get("/config-manager/v1/config/raw")
        .operation_id("config_manager.get_config_raw")
        .summary("Get raw YAML configuration")
        .description("Returns the raw YAML file content for Monaco editor display")
        .tag("config-manager")
        .require_auth(&Resource::Config, &Action::Read)
        .require_license_features::<License>([])
        .query_param("env", false, "Environment: production, testnet, testnet_scalping")
        .handler(handlers::get_config_raw)
        .json_response_with_schema::<dto::RawConfigResponse>(openapi, http::StatusCode::OK, "Raw YAML content")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    // PUT /config-manager/v1/config/raw
    router = OperationBuilder::put("/config-manager/v1/config/raw")
        .operation_id("config_manager.update_config_raw")
        .summary("Update raw YAML configuration")
        .description("Writes raw YAML content to config file after syntax validation")
        .tag("config-manager")
        .require_auth(&Resource::Config, &Action::Write)
        .require_license_features::<License>([])
        .query_param("env", false, "Environment: production, testnet, testnet_scalping")
        .json_request::<dto::RawConfigUpdateRequest>(openapi, "Raw YAML content")
        .handler(handlers::update_config_raw)
        .json_response_with_schema::<dto::SuccessResponse>(openapi, http::StatusCode::OK, "Success")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    // ================================================================
    // Trade pairs endpoints
    // ================================================================

    // GET /config-manager/v1/pairs
    router = OperationBuilder::get("/config-manager/v1/pairs")
        .operation_id("config_manager.get_trade_pairs")
        .summary("Get trade pairs with evolution data")
        .description("Returns all trading pairs with price evolution, open orders, and exclusion status")
        .tag("config-manager")
        .require_auth(&Resource::Config, &Action::Read)
        .require_license_features::<License>([])
        .handler(handlers::get_trade_pairs)
        .json_response_with_schema::<dto::TradePairsListDto>(openapi, http::StatusCode::OK, "Trade pairs")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    // GET /config-manager/v1/pairs/{symbol}/signals
    router = OperationBuilder::get("/config-manager/v1/pairs/{symbol}/signals")
        .operation_id("config_manager.get_signals")
        .summary("Get chart signals for a trading pair")
        .description("Returns OHLCV data with technical indicators (RSI, EMA, SMA, MACD, BB, etc.)")
        .tag("config-manager")
        .require_auth(&Resource::Config, &Action::Read)
        .require_license_features::<License>([])
        .path_param("symbol", "Trading pair symbol (e.g. BTCUSDC)")
        .query_param("env", false, "Environment")
        .query_param("percent", false, "Profit percentage (default: 1.0)")
        .handler(handlers::get_signals)
        .json_response_with_schema::<dto::SignalsResponse>(openapi, http::StatusCode::OK, "Signal data with indicators")
        .error_400(openapi).error_404(openapi).error_500(openapi)
        .register(router, openapi);

    // PUT /config-manager/v1/pairs/{symbol}/profit-percent
    router = OperationBuilder::put("/config-manager/v1/pairs/{symbol}/profit-percent")
        .operation_id("config_manager.update_profit_percent")
        .summary("Update profit target for a pair")
        .description("Updates the estimated profit percentage (signals_percent_difference) for a trading pair")
        .tag("config-manager")
        .require_auth(&Resource::Config, &Action::Write)
        .require_license_features::<License>([])
        .path_param("symbol", "Trading pair symbol")
        .json_request::<dto::ProfitPercentUpdateRequest>(openapi, "New profit percent")
        .handler(handlers::update_profit_percent)
        .json_response_with_schema::<dto::ProfitPercentUpdateResponse>(openapi, http::StatusCode::OK, "Update result")
        .error_400(openapi).error_404(openapi).error_500(openapi)
        .register(router, openapi);

    // ================================================================
    // Restart endpoint
    // ================================================================

    // POST /config-manager/v1/restart
    router = OperationBuilder::post("/config-manager/v1/restart")
        .operation_id("config_manager.restart_container")
        .summary("Restart trading container")
        .description("Restarts the Docker container for the specified environment")
        .tag("config-manager")
        .require_auth(&Resource::Config, &Action::Write)
        .require_license_features::<License>([])
        .query_param("env", false, "Environment: production, testnet")
        .handler(handlers::restart_container)
        .json_response_with_schema::<dto::RestartResponse>(openapi, http::StatusCode::OK, "Restart result")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    // Inject the service as Extension for all routes
    router.layer(axum::Extension(service))
}
