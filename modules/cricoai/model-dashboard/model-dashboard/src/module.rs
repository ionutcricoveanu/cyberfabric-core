use std::sync::Arc;

use async_trait::async_trait;
use modkit::api::OpenApiRegistry;
use modkit::{Module, ModuleCtx, RestApiCapability};
use tracing::info;

use model_dashboard_sdk::ModelDashboardApi;

use crate::api::rest::routes;
use crate::config::ModelDashboardConfig;
use crate::domain::local_client::ModelDashboardLocalClient;
use crate::domain::service::ModelDashboardService;

#[modkit::module(
    name = "model-dashboard",
    capabilities = [rest]
)]
pub struct ModelDashboard {
    service: arc_swap::ArcSwapOption<ModelDashboardService>,
}

impl Default for ModelDashboard {
    fn default() -> Self {
        Self {
            service: arc_swap::ArcSwapOption::from(None),
        }
    }
}

impl Clone for ModelDashboard {
    fn clone(&self) -> Self {
        Self {
            service: arc_swap::ArcSwapOption::new(self.service.load().as_ref().map(Clone::clone)),
        }
    }
}

#[async_trait]
impl Module for ModelDashboard {
    async fn init(&self, ctx: &ModuleCtx) -> anyhow::Result<()> {
        info!("Initializing model-dashboard module");

        let cfg: ModelDashboardConfig = ctx.config()?;

        // Connect to Model_Data database directly (raw SeaORM)
        let model_conn = if let Some(dsn) = &cfg.model_data_dsn {
            info!("Connecting to Model_Data database");
            sea_orm::Database::connect(dsn)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to connect to Model_Data: {e}"))?
        } else {
            return Err(anyhow::anyhow!(
                "model-dashboard requires model_data_dsn to be configured"
            ));
        };

        let service = Arc::new(ModelDashboardService::new(model_conn, cfg));
        self.service.store(Some(service.clone()));

        let local = ModelDashboardLocalClient::new(service);
        ctx.client_hub()
            .register::<dyn ModelDashboardApi>(Arc::new(local));

        info!("ModelDashboard client registered into ClientHub");
        Ok(())
    }
}

impl RestApiCapability for ModelDashboard {
    fn register_rest(
        &self,
        _ctx: &ModuleCtx,
        router: axum::Router,
        openapi: &dyn OpenApiRegistry,
    ) -> anyhow::Result<axum::Router> {
        info!("Registering model-dashboard REST routes");

        let service = self
            .service
            .load()
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Service not initialized"))?
            .clone();

        let router = routes::register_routes(router, openapi, service);

        info!("Model-dashboard REST routes registered successfully");
        Ok(router)
    }
}
