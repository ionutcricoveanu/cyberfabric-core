use std::sync::Arc;

use async_trait::async_trait;
use modkit::api::OpenApiRegistry;
use modkit::{Module, ModuleCtx, RestApiCapability};
use tracing::info;

use crate::api::rest::register_routes;
use crate::config::MarketDataConfig;
use crate::domain::local_client::MarketDataLocalClient;
use crate::domain::service::MarketDataService;
use market_data_sdk::MarketDataApi;

/// Main module struct for the market-data module with lifecycle management
#[modkit::module(
    name = "market-data",
    capabilities = [stateful, rest],
    lifecycle(entry = "run_collector", stop_timeout = "30s")
)]
pub struct MarketData {
    service: arc_swap::ArcSwapOption<MarketDataService>,
}

impl Default for MarketData {
    fn default() -> Self {
        Self {
            service: arc_swap::ArcSwapOption::from(None),
        }
    }
}

impl Clone for MarketData {
    fn clone(&self) -> Self {
        Self {
            service: arc_swap::ArcSwapOption::new(self.service.load().as_ref().map(Clone::clone)),
        }
    }
}

#[async_trait]
impl Module for MarketData {
    async fn init(&self, ctx: &ModuleCtx) -> anyhow::Result<()> {
        info!("Initializing market-data module");

        let cfg: MarketDataConfig = ctx.config()?;

        // Connect to Binance database (for reading trade_pairs)
        info!("Connecting to Binance database");
        let binance_db = modkit_db::connect_db(&cfg.binance_dsn, modkit_db::ConnectOpts::default())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect to Binance database: {e}"))?;

        // Connect to Binance_Klines database (for storing klines)
        info!("Connecting to Binance_Klines database");
        let klines_db = modkit_db::connect_db(&cfg.klines_dsn, modkit_db::ConnectOpts::default())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect to Binance_Klines database: {e}"))?;

        let service = MarketDataService::new(
            Arc::new(binance_db),
            Arc::new(klines_db),
            cfg,
        )?;

        self.service.store(Some(service.clone()));

        // Register in ClientHub for inter-module access
        let local = MarketDataLocalClient::new(service);
        ctx.client_hub()
            .register::<dyn MarketDataApi>(Arc::new(local));

        info!("MarketData client registered into ClientHub");
        info!("MarketData module initialized");
        Ok(())
    }
}

impl RestApiCapability for MarketData {
    fn register_rest(
        &self,
        _ctx: &ModuleCtx,
        router: axum::Router,
        openapi: &dyn OpenApiRegistry,
    ) -> anyhow::Result<axum::Router> {
        info!("Registering market-data REST routes");

        let service = self
            .service
            .load()
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Service not initialized"))?
            .clone();

        let router = register_routes(router, openapi, service);

        info!("Market-data REST routes registered successfully");
        Ok(router)
    }
}

impl MarketData {
    /// Lifecycle entry point - runs the kline collector
    pub(crate) async fn run_collector(
        self: Arc<Self>,
        cancel: tokio_util::sync::CancellationToken,
    ) -> anyhow::Result<()> {
        info!("Starting market-data collector lifecycle");

        let service = self
            .service
            .load()
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("MarketDataService not initialized"))?
            .clone();

        // Run the collector (blocks until cancelled)
        service.collector.run(cancel).await;

        info!("Market-data collector lifecycle completed");
        Ok(())
    }
}
