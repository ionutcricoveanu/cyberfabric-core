//! Module definition and lifecycle

use std::sync::Arc;

use async_trait::async_trait;
use data_ingestion_sdk::DataIngestionApi;
use modkit::api::OpenApiRegistry;
use modkit::{Module, ModuleCtx, RestApiCapability};
use tracing::info;

use crate::config::DataIngestionConfig;
use crate::domain::service::DataIngestionService;

/// Main module struct for the data-ingestion module with lifecycle management
#[modkit::module(
    name = "data-ingestion",
    capabilities = [stateful, rest],
    lifecycle(entry = "run_collector", stop_timeout = "30s")
)]
pub struct DataIngestionModule {
    service: arc_swap::ArcSwapOption<DataIngestionService>,
}

impl Default for DataIngestionModule {
    fn default() -> Self {
        Self {
            service: arc_swap::ArcSwapOption::from(None),
        }
    }
}

impl Clone for DataIngestionModule {
    fn clone(&self) -> Self {
        Self {
            service: arc_swap::ArcSwapOption::new(self.service.load().as_ref().map(Clone::clone)),
        }
    }
}

#[async_trait]
impl Module for DataIngestionModule {
    async fn init(&self, ctx: &ModuleCtx) -> anyhow::Result<()> {
        info!("Initializing data-ingestion module");

        let cfg: DataIngestionConfig = ctx.config()?;

        info!("Connecting to Model_Data database");
        let db = modkit_db::connect_db(&cfg.model_data_dsn, modkit_db::ConnectOpts::default())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect to Model_Data database: {e}"))?;

        let service = Arc::new(DataIngestionService::new(Arc::new(db), cfg.clone()));

        self.service.store(Some(service.clone()));

        // Register in ClientHub for inter-module access
        let local = DataIngestionLocalClient {
            service: service.clone(),
        };
        ctx.client_hub()
            .register::<dyn DataIngestionApi>(Arc::new(local));

        info!("Data-ingestion client registered into ClientHub");
        info!("Data-ingestion module initialized");
        Ok(())
    }
}

/// Local client adapter for in-process communication
struct DataIngestionLocalClient {
    service: Arc<DataIngestionService>,
}

#[async_trait::async_trait]
impl DataIngestionApi for DataIngestionLocalClient {
    async fn get_latest_sentiment(
        &self,
        ctx: &modkit_security::SecurityContext,
        symbol: &str,
    ) -> std::result::Result<Vec<data_ingestion_sdk::SentimentEntry>, data_ingestion_sdk::DataIngestionError>
    {
        self.service.get_latest_sentiment(ctx, symbol).await
    }
}

#[async_trait]
impl RestApiCapability for DataIngestionModule {
    fn register_rest(
        &self,
        _ctx: &ModuleCtx,
        router: axum::Router,
        _openapi: &dyn OpenApiRegistry,
    ) -> anyhow::Result<axum::Router> {
        info!("Registering data-ingestion REST routes");

        let service = self
            .service
            .load()
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Service not initialized"))?
            .clone();

        let router = router.nest(
            "/data-ingestion",
            crate::api::rest::routes::routes().layer(axum::Extension(service)),
        );

        info!("Data-ingestion REST routes registered successfully");
        Ok(router)
    }
}

impl DataIngestionModule {
    /// Lifecycle entry point - runs the collector
    pub(crate) async fn run_collector(
        self: Arc<Self>,
        cancel: tokio_util::sync::CancellationToken,
    ) -> anyhow::Result<()> {
        info!("Starting data-ingestion collector lifecycle");

        let service = self
            .service
            .load()
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("DataIngestionService not initialized"))?
            .clone();

        // Run the collector (blocks until cancelled)
        service.run_collector(cancel).await?;

        info!("Data-ingestion collector lifecycle completed");
        Ok(())
    }
}
