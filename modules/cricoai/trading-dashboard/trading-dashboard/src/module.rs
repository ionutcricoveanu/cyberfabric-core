use std::sync::Arc;

use async_trait::async_trait;
use modkit::api::OpenApiRegistry;
use modkit::{DatabaseCapability, Module, ModuleCtx, RestApiCapability};
use modkit_db::DBProvider;
use modkit_db::DbError;
use sea_orm_migration::MigrationTrait;
use tracing::info;

use trading_dashboard_sdk::TradingDashboardApi;
use tenant_resolver_sdk::TenantResolverGatewayClient;

use crate::api::rest::routes;
use crate::config::TradingDashboardConfig;
use crate::domain::local_client::TradingDashboardLocalClient;
use crate::domain::service::TradingDashboardService;

/// Main module struct for the trading-dashboard module.
#[modkit::module(
    name = "trading-dashboard",
    deps = ["tenant-resolver"],
    capabilities = [db, rest]
)]
pub struct TradingDashboard {
    service: arc_swap::ArcSwapOption<TradingDashboardService>,
}

impl Default for TradingDashboard {
    fn default() -> Self {
        Self {
            service: arc_swap::ArcSwapOption::from(None),
        }
    }
}

impl Clone for TradingDashboard {
    fn clone(&self) -> Self {
        Self {
            service: arc_swap::ArcSwapOption::new(self.service.load().as_ref().map(Clone::clone)),
        }
    }
}

#[async_trait]
impl Module for TradingDashboard {
    async fn init(&self, ctx: &ModuleCtx) -> anyhow::Result<()> {
        info!("Initializing trading-dashboard module");

        let cfg: TradingDashboardConfig = ctx.config()?;

        let db: Arc<DBProvider<DbError>> = Arc::new(ctx.db_required()?);

        let resolver = ctx
            .client_hub()
            .get::<dyn TenantResolverGatewayClient>()
            .map_err(|e| anyhow::anyhow!("failed to get tenant resolver: {e}"))?;

        let service = Arc::new(TradingDashboardService::new(db, resolver, cfg));
        self.service.store(Some(service.clone()));

        let local = TradingDashboardLocalClient::new(service);
        ctx.client_hub()
            .register::<dyn TradingDashboardApi>(Arc::new(local));

        info!("TradingDashboard client registered into ClientHub");
        Ok(())
    }
}

impl DatabaseCapability for TradingDashboard {
    fn migrations(&self) -> Vec<Box<dyn MigrationTrait>> {
        // No migrations yet — we're reading existing tables.
        // Migrations to add tenant_id will come later.
        vec![]
    }
}

impl RestApiCapability for TradingDashboard {
    fn register_rest(
        &self,
        _ctx: &ModuleCtx,
        router: axum::Router,
        openapi: &dyn OpenApiRegistry,
    ) -> anyhow::Result<axum::Router> {
        info!("Registering trading-dashboard REST routes");

        let service = self
            .service
            .load()
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Service not initialized"))?
            .clone();

        let router = routes::register_routes(router, openapi, service);

        info!("Trading dashboard REST routes registered successfully");
        Ok(router)
    }
}
