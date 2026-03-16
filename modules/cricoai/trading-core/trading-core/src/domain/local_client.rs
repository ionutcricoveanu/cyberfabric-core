//! Local client implementation for inter-module communication

use async_trait::async_trait;
use modkit_security::SecurityContext;
use std::sync::Arc;
use trading_core_sdk::{TradingCoreApi, BotStatus, TradingCoreError};

use crate::domain::service::TradingCoreService;

/// Local client adapter for ClientHub registration
pub struct LocalClient {
    service: Arc<TradingCoreService>,
}

impl LocalClient {
    /// Create a new local client
    pub fn new(service: Arc<TradingCoreService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl TradingCoreApi for LocalClient {
    async fn get_status(
        &self,
        ctx: &SecurityContext,
    ) -> Result<BotStatus, TradingCoreError> {
        self.service.get_status(ctx).await
    }

    async fn pause(
        &self,
        ctx: &SecurityContext,
    ) -> Result<(), TradingCoreError> {
        self.service.pause(ctx).await
    }

    async fn resume(
        &self,
        ctx: &SecurityContext,
    ) -> Result<(), TradingCoreError> {
        self.service.resume(ctx).await
    }
}
