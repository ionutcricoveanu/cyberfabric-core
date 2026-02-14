//! Object-safe client trait for the trading-dashboard module.
//!
//! This API is designed for `ClientHub` registration as `Arc<dyn TradingDashboardApi>`.

use async_trait::async_trait;
use modkit_security::SecurityContext;

use crate::errors::TradingDashboardError;
use crate::models::StatsSummary;

/// Object-safe client for inter-module consumption (`ClientHub` registered).
///
/// ```ignore
/// let dashboard = hub.get::<dyn TradingDashboardApi>()?;
/// let summary = dashboard.get_stats_summary(&ctx, "24h").await?;
/// ```
#[async_trait]
pub trait TradingDashboardApi: Send + Sync {
    /// Get aggregated trading statistics summary.
    async fn get_stats_summary(
        &self,
        ctx: &SecurityContext,
        time_range: &str,
    ) -> Result<StatsSummary, TradingDashboardError>;
}
