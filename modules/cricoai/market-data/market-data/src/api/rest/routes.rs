use std::sync::Arc;

use axum::Router;
use modkit::api::OpenApiRegistry;
use modkit::api::operation_builder::{AuthReqAction, AuthReqResource, LicenseFeature, OperationBuilder};

use crate::api::rest::handlers;
use crate::domain::service::MarketDataService;

// Authorization enums for market-data routes

pub(crate) enum Resource {
    Klines,
    Ticker,
}

pub(crate) enum Action {
    Read,
}

impl AsRef<str> for Resource {
    fn as_ref(&self) -> &'static str {
        match self {
            Self::Klines => "klines",
            Self::Ticker => "ticker",
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
    service: Arc<MarketDataService>,
) -> Router {
    // GET /market-data/v1/klines/{symbol}
    router = OperationBuilder::get("/market-data/v1/klines/{symbol}")
        .operation_id("market_data.get_klines")
        .summary("Get klines for a symbol")
        .description("Retrieve historical kline (candlestick) data for a trading pair")
        .tag("market-data")
        .require_auth(&Resource::Klines, &Action::Read)
        .require_license_features::<License>([])
        .query_param("interval", true, "Kline interval (1m, 5m, 15m, 1h, 4h)")
        .query_param("limit", false, "Number of klines to return (default: 100, max: 1000)")
        .handler(handlers::get_klines)
        .json_response_with_schema::<handlers::KlinesResponse>(
            openapi,
            http::StatusCode::OK,
            "Klines data",
        )
        .error_400(openapi)
        .error_500(openapi)
        .register(router, openapi);

    // GET /market-data/v1/ticker/{symbol}
    router = OperationBuilder::get("/market-data/v1/ticker/{symbol}")
        .operation_id("market_data.get_ticker_24h")
        .summary("Get 24h ticker for a symbol")
        .description("Retrieve 24-hour ticker statistics for a trading pair")
        .tag("market-data")
        .require_auth(&Resource::Ticker, &Action::Read)
        .require_license_features::<License>([])
        .handler(handlers::get_ticker_24h)
        .json_response_with_schema::<handlers::Ticker24hResponse>(
            openapi,
            http::StatusCode::OK,
            "24h ticker statistics",
        )
        .error_400(openapi)
        .error_500(openapi)
        .register(router, openapi);

    // GET /market-data/v1/tickers
    router = OperationBuilder::get("/market-data/v1/tickers")
        .operation_id("market_data.get_all_tickers_24h")
        .summary("Get 24h tickers for all symbols")
        .description("Retrieve 24-hour ticker statistics for all trading pairs")
        .tag("market-data")
        .require_auth(&Resource::Ticker, &Action::Read)
        .require_license_features::<License>([])
        .handler(handlers::get_all_tickers_24h)
        .json_response_with_schema::<handlers::AllTickers24hResponse>(
            openapi,
            http::StatusCode::OK,
            "List of 24h ticker statistics",
        )
        .error_400(openapi)
        .error_500(openapi)
        .register(router, openapi);

    // Inject the service as Extension for all routes
    router.layer(axum::Extension(service))
}
