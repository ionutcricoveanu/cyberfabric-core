use std::sync::Arc;

use async_trait::async_trait;
use modkit_security::SecurityContext;

use agent_trade_impact_sdk::{AgentTradeImpactApi, AgentTradeImpactError};
use crate::domain::service::AgentTradeImpactService;

pub struct AgentTradeImpactLocalClient {
    service: Arc<AgentTradeImpactService>,
}

impl AgentTradeImpactLocalClient {
    pub fn new(service: Arc<AgentTradeImpactService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl AgentTradeImpactApi for AgentTradeImpactLocalClient {
    async fn health_check(&self, ctx: &SecurityContext) -> Result<(), AgentTradeImpactError> {
        self.service
            .get_effectiveness(ctx, "production", 1)
            .await
            .map_err(|e| AgentTradeImpactError::Database(e.to_string()))?;
        Ok(())
    }
}
