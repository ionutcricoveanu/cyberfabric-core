use async_trait::async_trait;
use modkit_security::SecurityContext;

use crate::errors::AgentTradeImpactError;

#[async_trait]
pub trait AgentTradeImpactApi: Send + Sync + 'static {
    async fn health_check(&self, ctx: &SecurityContext) -> Result<(), AgentTradeImpactError>;
}
