use std::sync::Arc;

use async_trait::async_trait;
use modkit::api::OpenApiRegistry;
use modkit::{Module, ModuleCtx, RestApiCapability};
use tracing::info;

use performance_monitor_sdk::PerformanceMonitorApi;

use crate::api::rest::routes;
use crate::config::PerformanceMonitorConfig;
use crate::domain::local_client::PerformanceMonitorLocalClient;
use crate::domain::service::PerformanceMonitorService;

#[modkit::module(
    name = "performance-monitor",
    capabilities = [rest]
)]
pub struct PerformanceMonitor {
    service: arc_swap::ArcSwapOption<PerformanceMonitorService>,
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self {
            service: arc_swap::ArcSwapOption::from(None),
        }
    }
}

impl Clone for PerformanceMonitor {
    fn clone(&self) -> Self {
        Self {
            service: arc_swap::ArcSwapOption::new(self.service.load().as_ref().map(Clone::clone)),
        }
    }
}

#[async_trait]
impl Module for PerformanceMonitor {
    async fn init(&self, ctx: &ModuleCtx) -> anyhow::Result<()> {
        info!("Initializing performance-monitor module");

        let cfg: PerformanceMonitorConfig = ctx.config()?;

        let prod_db = if let Some(dsn) = &cfg.binance_dsn {
            info!("Connecting to Binance (production) database");
            Some(
                sea_orm::Database::connect(dsn)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to connect to Binance: {e}"))?,
            )
        } else {
            None
        };

        let testnet_db = if let Some(dsn) = &cfg.binance_tn_dsn {
            info!("Connecting to BinanceTN (testnet) database");
            Some(
                sea_orm::Database::connect(dsn)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to connect to BinanceTN: {e}"))?,
            )
        } else {
            None
        };

        if prod_db.is_none() && testnet_db.is_none() {
            return Err(anyhow::anyhow!(
                "performance-monitor requires at least one of binance_dsn or binance_tn_dsn"
            ));
        }

        let service = Arc::new(PerformanceMonitorService::new(prod_db, testnet_db, cfg));
        self.service.store(Some(service.clone()));

        let local = PerformanceMonitorLocalClient::new(service);
        ctx.client_hub()
            .register::<dyn PerformanceMonitorApi>(Arc::new(local));

        info!("PerformanceMonitor client registered into ClientHub");
        Ok(())
    }
}

impl RestApiCapability for PerformanceMonitor {
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
