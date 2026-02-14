use std::sync::Arc;

use async_trait::async_trait;
use modkit_security::SecurityContext;

use trading_dashboard_sdk::errors::TradingDashboardError;
use trading_dashboard_sdk::models::StatsSummary;
use trading_dashboard_sdk::TradingDashboardApi;

use crate::domain::service::TradingDashboardService;

/// Local (in-process) adapter implementing the SDK trait for ClientHub registration.
pub struct TradingDashboardLocalClient {
    service: Arc<TradingDashboardService>,
}

impl TradingDashboardLocalClient {
    #[must_use]
    pub fn new(service: Arc<TradingDashboardService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl TradingDashboardApi for TradingDashboardLocalClient {
    async fn get_stats_summary(
        &self,
        ctx: &SecurityContext,
        time_range: &str,
    ) -> Result<StatsSummary, TradingDashboardError> {
        self.service.get_stats_summary(ctx, time_range).await
    }
}
