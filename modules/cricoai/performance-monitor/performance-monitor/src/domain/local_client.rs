use std::sync::Arc;

use async_trait::async_trait;
use modkit_security::SecurityContext;

use performance_monitor_sdk::{PerformanceMonitorApi, PerformanceMonitorError, PerformanceOverview};
use crate::domain::service::PerformanceMonitorService;

pub struct PerformanceMonitorLocalClient {
    service: Arc<PerformanceMonitorService>,
}

impl PerformanceMonitorLocalClient {
    pub fn new(service: Arc<PerformanceMonitorService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl PerformanceMonitorApi for PerformanceMonitorLocalClient {
    async fn get_overview(
        &self,
        ctx: &SecurityContext,
    ) -> Result<PerformanceOverview, PerformanceMonitorError> {
        let resp = self
            .service
            .get_overview(ctx, "production", 24)
            .await
            .map_err(|e| PerformanceMonitorError::Database(e.to_string()))?;

        Ok(PerformanceOverview {
            total_trades: resp.total_trades,
            win_rate: resp.win_rate,
            total_profit_usdc: resp.total_profit_usdc,
        })
    }
}
