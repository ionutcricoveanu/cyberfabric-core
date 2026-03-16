//! Request/Response DTOs

use modkit_macros::api_dto;
use serde::{Deserialize, Serialize};

/// Response: Agent decision for a symbol
#[api_dto(response)]
#[derive(Debug, Clone)]
pub struct AgentDecisionDto {
    pub symbol: String,
    pub action: String,
    pub confidence: f64,
    pub reasoning: String,
    pub created_at: String,
}

/// Response: Latest agent decisions
#[api_dto(response)]
#[derive(Debug, Clone)]
pub struct LatestDecisionsResponse {
    pub decisions: Vec<AgentDecisionDto>,
}

/// Response: Scheduler status
#[api_dto(response)]
#[derive(Debug, Clone)]
pub struct SchedulerStatusResponse {
    pub running: bool,
    pub tier_intervals: TierIntervalsDto,
}

/// Tier interval configuration
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct TierIntervalsDto {
    pub hot_minutes: u64,
    pub warm_minutes: u64,
    pub stable_minutes: u64,
    pub cold_minutes: u64,
}
