use async_trait::async_trait;
use modkit_security::SecurityContext;

use crate::errors::ModelDashboardError;
use crate::models::ModelSummary;

#[async_trait]
pub trait ModelDashboardApi: Send + Sync + 'static {
    async fn get_model_summary(
        &self,
        ctx: &SecurityContext,
    ) -> Result<ModelSummary, ModelDashboardError>;
}
