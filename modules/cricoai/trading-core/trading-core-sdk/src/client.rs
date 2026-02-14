//! Object-safe client trait for the trading-core module.

use async_trait::async_trait;
use modkit_security::SecurityContext;

use crate::errors::TradingCoreError;
use crate::models::BotStatus;

#[async_trait]
pub trait TradingCoreApi: Send + Sync {
    /// Get the current bot status.
    async fn get_status(
        &self,
        ctx: &SecurityContext,
    ) -> Result<BotStatus, TradingCoreError>;

    /// Pause the trading loop.
    async fn pause(
        &self,
        ctx: &SecurityContext,
    ) -> Result<(), TradingCoreError>;

    /// Resume the trading loop.
    async fn resume(
        &self,
        ctx: &SecurityContext,
    ) -> Result<(), TradingCoreError>;
}
