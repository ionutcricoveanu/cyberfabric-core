use std::sync::Arc;

use async_trait::async_trait;
use modkit::api::OpenApiRegistry;
use modkit::{Module, ModuleCtx, RestApiCapability};
use tracing::info;

use auth_management_sdk::AuthManagementApi;

use crate::api::rest::routes;
use crate::config::AuthManagementConfig;
use crate::domain::local_client::AuthManagementLocalClient;
use crate::domain::service::AuthManagementService;

#[modkit::module(
    name = "auth-management",
    capabilities = [rest]
)]
pub struct AuthManagement {
    service: arc_swap::ArcSwapOption<AuthManagementService>,
}

impl Default for AuthManagement {
    fn default() -> Self {
        Self {
            service: arc_swap::ArcSwapOption::from(None),
        }
    }
}

impl Clone for AuthManagement {
    fn clone(&self) -> Self {
        Self {
            service: arc_swap::ArcSwapOption::new(self.service.load().as_ref().map(Clone::clone)),
        }
    }
}

#[async_trait]
impl Module for AuthManagement {
    async fn init(&self, ctx: &ModuleCtx) -> anyhow::Result<()> {
        info!("Initializing auth-management module");

        let cfg: AuthManagementConfig = ctx.config()?;

        let db_conn = if let Some(dsn) = &cfg.binance_dsn {
            info!("Connecting to Binance database (auth tables)");
            sea_orm::Database::connect(dsn)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to connect to Binance: {e}"))?
        } else {
            return Err(anyhow::anyhow!(
                "auth-management requires binance_dsn to be configured"
            ));
        };

        let service = Arc::new(AuthManagementService::new(db_conn, cfg));
        self.service.store(Some(service.clone()));

        let local = AuthManagementLocalClient::new(service);
        ctx.client_hub()
            .register::<dyn AuthManagementApi>(Arc::new(local));

        info!("AuthManagement client registered into ClientHub");
        Ok(())
    }
}

impl RestApiCapability for AuthManagement {
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
