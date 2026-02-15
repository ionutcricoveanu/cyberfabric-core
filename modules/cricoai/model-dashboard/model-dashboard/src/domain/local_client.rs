use std::sync::Arc;

use async_trait::async_trait;
use modkit_security::SecurityContext;

use model_dashboard_sdk::{ModelDashboardApi, ModelDashboardError, ModelSummary};
use crate::domain::service::ModelDashboardService;

pub struct ModelDashboardLocalClient {
    service: Arc<ModelDashboardService>,
}

impl ModelDashboardLocalClient {
    pub fn new(service: Arc<ModelDashboardService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl ModelDashboardApi for ModelDashboardLocalClient {
    async fn get_model_summary(
        &self,
        ctx: &SecurityContext,
    ) -> Result<ModelSummary, ModelDashboardError> {
        let resp = self
            .service
            .get_summary(ctx, "all", "7d")
            .await
            .map_err(|e| ModelDashboardError::Database(e.to_string()))?;

        Ok(ModelSummary {
            total_trainings: resp.totals.total_trainings,
            total_promoted: resp.totals.total_promoted,
            unique_symbols: resp.totals.unique_symbols,
            overall_promotion_rate: resp.totals.overall_promotion_rate,
        })
    }
}
