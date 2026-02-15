use std::sync::Arc;

use async_trait::async_trait;
use modkit::api::OpenApiRegistry;
use modkit::{Module, ModuleCtx, RestApiCapability};
use tracing::info;

use agent_trade_impact_sdk::AgentTradeImpactApi;

use crate::api::rest::routes;
use crate::config::AgentTradeImpactConfig;
use crate::domain::local_client::AgentTradeImpactLocalClient;
use crate::domain::service::AgentTradeImpactService;

#[modkit::module(
    name = "agent-trade-impact",
    capabilities = [rest]
)]
pub struct AgentTradeImpact {
    service: arc_swap::ArcSwapOption<AgentTradeImpactService>,
}

impl Default for AgentTradeImpact {
    fn default() -> Self {
        Self {
            service: arc_swap::ArcSwapOption::from(None),
        }
    }
}

impl Clone for AgentTradeImpact {
    fn clone(&self) -> Self {
        Self {
            service: arc_swap::ArcSwapOption::new(self.service.load().as_ref().map(Clone::clone)),
        }
    }
}

#[async_trait]
impl Module for AgentTradeImpact {
    async fn init(&self, ctx: &ModuleCtx) -> anyhow::Result<()> {
        info!("Initializing agent-trade-impact module");

        let cfg: AgentTradeImpactConfig = ctx.config()?;

        let model_conn = if let Some(dsn) = &cfg.model_data_dsn {
            info!("Connecting to Model_Data database");
            sea_orm::Database::connect(dsn)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to connect to Model_Data: {e}"))?
        } else {
            return Err(anyhow::anyhow!(
                "agent-trade-impact requires model_data_dsn to be configured"
            ));
        };

        let service = Arc::new(AgentTradeImpactService::new(model_conn, cfg));
        self.service.store(Some(service.clone()));

        let local = AgentTradeImpactLocalClient::new(service);
        ctx.client_hub()
            .register::<dyn AgentTradeImpactApi>(Arc::new(local));

        info!("AgentTradeImpact client registered into ClientHub");
        Ok(())
    }
}

impl RestApiCapability for AgentTradeImpact {
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
