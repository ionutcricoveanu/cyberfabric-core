use std::sync::Arc;

use async_trait::async_trait;
use modkit_security::SecurityContext;

use agent_analytics_sdk::{AgentAnalyticsApi, AgentAnalyticsError};
use crate::domain::service::AgentAnalyticsService;

pub struct AgentAnalyticsLocalClient {
    service: Arc<AgentAnalyticsService>,
}

impl AgentAnalyticsLocalClient {
    pub fn new(service: Arc<AgentAnalyticsService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl AgentAnalyticsApi for AgentAnalyticsLocalClient {
    async fn health_check(&self, ctx: &SecurityContext) -> Result<(), AgentAnalyticsError> {
        self.service
            .get_overview(ctx)
            .await
            .map_err(|e| AgentAnalyticsError::Database(e.to_string()))?;
        Ok(())
    }
}
