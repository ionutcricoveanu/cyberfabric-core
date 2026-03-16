//! Module definition and lifecycle

use std::sync::Arc;

use async_trait::async_trait;
use market_intelligence_sdk::MarketIntelligenceApi;
use modkit::api::OpenApiRegistry;
use modkit::{Module, ModuleCtx, RestApiCapability};
use tracing::info;

use crate::config::MarketIntelligenceConfig;
use crate::domain::service::MarketIntelligenceService;

/// Main module struct for the market-intelligence module with lifecycle management
#[modkit::module(
    name = "market-intelligence",
    capabilities = [stateful, rest],
    lifecycle(entry = "run_agent_scheduler", stop_timeout = "30s")
)]
pub struct MarketIntelligenceModule {
    service: arc_swap::ArcSwapOption<MarketIntelligenceService>,
}

impl Default for MarketIntelligenceModule {
    fn default() -> Self {
        Self {
            service: arc_swap::ArcSwapOption::from(None),
        }
    }
}

impl Clone for MarketIntelligenceModule {
    fn clone(&self) -> Self {
        Self {
            service: arc_swap::ArcSwapOption::new(self.service.load().as_ref().map(Clone::clone)),
        }
    }
}

#[async_trait]
impl Module for MarketIntelligenceModule {
    async fn init(&self, ctx: &ModuleCtx) -> anyhow::Result<()> {
        info!("Initializing market-intelligence module");

        let cfg: MarketIntelligenceConfig = ctx.config()?;

        info!("Connecting to Model_Data database");
        let db = modkit_db::connect_db(&cfg.model_data_dsn, modkit_db::ConnectOpts::default())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect to Model_Data database: {e}"))?;

        let service = Arc::new(MarketIntelligenceService::new(Arc::new(db), cfg.clone()));

        self.service.store(Some(service.clone()));

        // Register in ClientHub for inter-module access
        let local = MarketIntelligenceLocalClient {
            service: service.clone(),
        };
        ctx.client_hub()
            .register::<dyn MarketIntelligenceApi>(Arc::new(local));

        info!("Market-intelligence client registered into ClientHub");
        info!("Market-intelligence module initialized");
        Ok(())
    }
}

/// Local client adapter for in-process communication
struct MarketIntelligenceLocalClient {
    service: Arc<MarketIntelligenceService>,
}

#[async_trait::async_trait]
impl MarketIntelligenceApi for MarketIntelligenceLocalClient {
    async fn get_latest_decision(
        &self,
        ctx: &modkit_security::SecurityContext,
        symbol: &str,
    ) -> std::result::Result<Option<market_intelligence_sdk::AgentDecision>, market_intelligence_sdk::MarketIntelligenceError>
    {
        self.service.get_latest_decision(ctx, symbol).await
    }
}

#[async_trait]
impl RestApiCapability for MarketIntelligenceModule {
    fn register_rest(
        &self,
        _ctx: &ModuleCtx,
        router: axum::Router,
        _openapi: &dyn OpenApiRegistry,
    ) -> anyhow::Result<axum::Router> {
        info!("Registering market-intelligence REST routes");

        let service = self
            .service
            .load()
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Service not initialized"))?
            .clone();

        let router = router.nest(
            "/market-intelligence",
            crate::api::rest::routes::routes().layer(axum::Extension(service)),
        );

        info!("Market-intelligence REST routes registered successfully");
        Ok(router)
    }
}

impl MarketIntelligenceModule {
    /// Lifecycle entry point - runs the agent scheduler
    pub(crate) async fn run_agent_scheduler(
        self: Arc<Self>,
        cancel: tokio_util::sync::CancellationToken,
    ) -> anyhow::Result<()> {
        info!("Starting market-intelligence scheduler lifecycle");

        let service = self
            .service
            .load()
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("MarketIntelligenceService not initialized"))?
            .clone();

        // Run the scheduler (blocks until cancelled)
        service.run_scheduler(cancel).await?;

        info!("Market-intelligence scheduler lifecycle completed");
        Ok(())
    }
}
