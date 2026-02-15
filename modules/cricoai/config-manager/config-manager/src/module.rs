use std::sync::Arc;

use async_trait::async_trait;
use modkit::api::OpenApiRegistry;
use modkit::{DatabaseCapability, Module, ModuleCtx, RestApiCapability};
use modkit_db::DBProvider;
use modkit_db::DbError;
use sea_orm_migration::MigrationTrait;
use tracing::info;

use config_manager_sdk::ConfigManagerApi;

use crate::api::rest::routes;
use crate::config::ConfigManagerConfig;
use crate::domain::local_client::ConfigManagerLocalClient;
use crate::domain::service::ConfigManagerService;

#[modkit::module(
    name = "config-manager",
    capabilities = [db, rest]
)]
pub struct ConfigManager {
    service: arc_swap::ArcSwapOption<ConfigManagerService>,
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self {
            service: arc_swap::ArcSwapOption::from(None),
        }
    }
}

impl Clone for ConfigManager {
    fn clone(&self) -> Self {
        Self {
            service: arc_swap::ArcSwapOption::new(self.service.load().as_ref().map(Clone::clone)),
        }
    }
}

#[async_trait]
impl Module for ConfigManager {
    async fn init(&self, ctx: &ModuleCtx) -> anyhow::Result<()> {
        info!("Initializing config-manager module");

        let cfg: ConfigManagerConfig = ctx.config()?;

        let db: Arc<DBProvider<DbError>> = Arc::new(ctx.db_required()?);

        // Connect to Binance_Klines database directly (raw SeaORM) for dynamic table queries
        let klines_conn = if let Some(dsn) = &cfg.klines_dsn {
            info!("Connecting to Binance_Klines database");
            let conn = sea_orm::Database::connect(dsn)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to connect to Binance_Klines: {e}"))?;
            Some(conn)
        } else {
            info!("No klines_dsn configured — signals endpoint will be unavailable");
            None
        };

        let service = Arc::new(ConfigManagerService::new(db, klines_conn, cfg));
        self.service.store(Some(service.clone()));

        let local = ConfigManagerLocalClient::new(service);
        ctx.client_hub()
            .register::<dyn ConfigManagerApi>(Arc::new(local));

        info!("ConfigManager client registered into ClientHub");
        Ok(())
    }
}

impl DatabaseCapability for ConfigManager {
    fn migrations(&self) -> Vec<Box<dyn MigrationTrait>> {
        vec![]
    }
}

impl RestApiCapability for ConfigManager {
    fn register_rest(
        &self,
        _ctx: &ModuleCtx,
        router: axum::Router,
        openapi: &dyn OpenApiRegistry,
    ) -> anyhow::Result<axum::Router> {
        info!("Registering config-manager REST routes");

        let service = self
            .service
            .load()
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Service not initialized"))?
            .clone();

        let router = routes::register_routes(router, openapi, service);

        info!("Config-manager REST routes registered successfully");
        Ok(router)
    }
}
