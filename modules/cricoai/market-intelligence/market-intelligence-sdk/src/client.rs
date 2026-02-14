//! Object-safe client trait for the market-intelligence module.

use async_trait::async_trait;
use modkit_security::SecurityContext;

use crate::errors::MarketIntelligenceError;
use crate::models::AgentDecision;

#[async_trait]
pub trait MarketIntelligenceApi: Send + Sync {
    /// Get the latest agent decision for a symbol.
    async fn get_latest_decision(
        &self,
        ctx: &SecurityContext,
        symbol: &str,
    ) -> Result<Option<AgentDecision>, MarketIntelligenceError>;
}
