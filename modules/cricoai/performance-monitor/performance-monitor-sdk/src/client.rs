use async_trait::async_trait;
use modkit_security::SecurityContext;

use crate::errors::PerformanceMonitorError;
use crate::models::PerformanceOverview;

#[async_trait]
pub trait PerformanceMonitorApi: Send + Sync + 'static {
    async fn get_overview(
        &self,
        ctx: &SecurityContext,
    ) -> Result<PerformanceOverview, PerformanceMonitorError>;
}
