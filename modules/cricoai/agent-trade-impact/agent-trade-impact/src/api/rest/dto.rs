// ================================================================
// Buy impact endpoint
// ================================================================

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct BuyImpactDto {
    pub pair: String,
    pub total_impacts: i64,
    pub avg_confidence: Option<f64>,
    pub buys_executed: i64,
    pub buys_blocked: i64,
    pub buys_modified: i64,
    pub avg_profit_pct: Option<f64>,
    pub avg_counterfactual_profit: Option<f64>,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct BuyImpactResponse {
    pub environment: String,
    pub period_days: i64,
    pub buy_impacts: Vec<BuyImpactDto>,
}

// ================================================================
// Sell impact endpoint
// ================================================================

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct SellImpactDto {
    pub pair: String,
    pub total_agent_exits: i64,
    pub avg_confidence: Option<f64>,
    pub avg_actual_profit: Option<f64>,
    pub avg_counterfactual_profit: Option<f64>,
    pub avg_agent_value_add: Option<f64>,
    pub profitable_exits: i64,
    pub loss_exits: i64,
    pub last_agent_exit: Option<String>,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct SellImpactResponse {
    pub environment: String,
    pub period_days: i64,
    pub sell_impacts: Vec<SellImpactDto>,
}

// ================================================================
// Effectiveness endpoint
// ================================================================

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct EffectivenessSummaryDto {
    pub total_interventions: i64,
    pub pairs_affected: i64,
    pub avg_confidence: Option<f64>,
    pub buy_interventions: i64,
    pub sell_interventions: i64,
    pub avg_actual_profit: Option<f64>,
    pub avg_counterfactual_profit: Option<f64>,
    pub avg_value_added: Option<f64>,
    pub times_agent_helped: i64,
    pub times_agent_hurt: i64,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct ActionBreakdownDto {
    pub action_taken: String,
    pub count: i64,
    pub avg_profit: Option<f64>,
    pub avg_counterfactual: Option<f64>,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct EffectivenessResponse {
    pub environment: String,
    pub period_days: i64,
    pub summary: EffectivenessSummaryDto,
    pub by_action: Vec<ActionBreakdownDto>,
}
