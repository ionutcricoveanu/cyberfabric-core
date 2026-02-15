use std::sync::Arc;

use async_trait::async_trait;
use modkit::api::OpenApiRegistry;
use modkit::{Module, ModuleCtx, RestApiCapability};
use tracing::info;

use agent_analytics_sdk::AgentAnalyticsApi;

use crate::api::rest::routes;
use crate::config::AgentAnalyticsConfig;
use crate::domain::local_client::AgentAnalyticsLocalClient;
use crate::domain::service::AgentAnalyticsService;

#[modkit::module(
    name = "agent-analytics",
    capabilities = [rest]
)]
pub struct AgentAnalytics {
    service: arc_swap::ArcSwapOption<AgentAnalyticsService>,
}

impl Default for AgentAnalytics {
    fn default() -> Self {
        Self {
            service: arc_swap::ArcSwapOption::from(None),
        }
    }
}

impl Clone for AgentAnalytics {
    fn clone(&self) -> Self {
        Self {
            service: arc_swap::ArcSwapOption::new(self.service.load().as_ref().map(Clone::clone)),
        }
    }
}

#[async_trait]
impl Module for AgentAnalytics {
    async fn init(&self, ctx: &ModuleCtx) -> anyhow::Result<()> {
        info!("Initializing agent-analytics module");

        let cfg: AgentAnalyticsConfig = ctx.config()?;

        let model_conn = if let Some(dsn) = &cfg.model_data_dsn {
            info!("Connecting to Model_Data database");
            sea_orm::Database::connect(dsn)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to connect to Model_Data: {e}"))?
        } else {
            return Err(anyhow::anyhow!(
                "agent-analytics requires model_data_dsn to be configured"
            ));
        };

        let service = Arc::new(AgentAnalyticsService::new(model_conn, cfg));
        self.service.store(Some(service.clone()));

        let local = AgentAnalyticsLocalClient::new(service);
        ctx.client_hub()
            .register::<dyn AgentAnalyticsApi>(Arc::new(local));

        info!("AgentAnalytics client registered into ClientHub");
        Ok(())
    }
}

impl RestApiCapability for AgentAnalytics {
    fn register_rest(
        &self,
        _ctx: &ModuleCtx,
        router: axum::Router,
        openapi: &dyn OpenApiRegistry,
    ) -> anyhow::Result<axum::Router> {
        let service = self
            .service
            .load()
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Service not initialized"))?
            .clone();

        let router = routes::register_routes(router, openapi, service);
        Ok(router)
    }
}
