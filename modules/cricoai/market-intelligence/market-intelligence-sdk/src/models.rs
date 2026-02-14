//! Domain models for the market-intelligence module.
//!
//! Transport-agnostic (no serde, no HTTP types).

/// An AI agent's sentiment decision for a trading symbol.
pub struct AgentDecision {
    pub symbol: String,
    pub action: String,
    pub confidence: f64,
    pub reasoning: String,
    pub created_at: chrono::NaiveDateTime,
}
