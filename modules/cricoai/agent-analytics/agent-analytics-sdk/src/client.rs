use async_trait::async_trait;
use modkit_security::SecurityContext;

use crate::errors::AgentAnalyticsError;

#[async_trait]
pub trait AgentAnalyticsApi: Send + Sync + 'static {
    async fn health_check(&self, ctx: &SecurityContext) -> Result<(), AgentAnalyticsError>;
}
