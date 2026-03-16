//! REST API handlers

use axum::extract::Path;
use axum::Extension;
use modkit::api::prelude::*;
use modkit_security::SecurityContext;
use std::sync::Arc;

use crate::domain::service::MarketIntelligenceService;

use super::dto::*;

/// Get latest decision for a symbol
pub async fn get_latest_decision(
    Extension(_ctx): Extension<SecurityContext>,
    Extension(_service): Extension<Arc<MarketIntelligenceService>>,
    Path(_symbol): Path<String>,
) -> ApiResult<JsonBody<AgentDecisionDto>> {
    // TODO: Retrieve from service and return actual decision
    // For now, return placeholder
    let decision = AgentDecisionDto {
        symbol: "BTC".to_string(),
        action: "HOLD".to_string(),
        confidence: 0.65,
        reasoning: "Waiting for sentiment data".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    Ok(Json(decision))
}

/// Get scheduler status
pub async fn get_status(
    Extension(_service): Extension<Arc<MarketIntelligenceService>>,
) -> ApiResult<JsonBody<SchedulerStatusResponse>> {
    let status = SchedulerStatusResponse {
        running: true,
        tier_intervals: TierIntervalsDto {
            hot_minutes: 5,
            warm_minutes: 15,
            stable_minutes: 30,
            cold_minutes: 60,
        },
    };

    Ok(Json(status))
}
