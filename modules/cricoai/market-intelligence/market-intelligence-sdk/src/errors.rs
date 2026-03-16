//! Error types for the market-intelligence module.

#[derive(Debug, thiserror::Error)]
pub enum MarketIntelligenceError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("database error: {0}")]
    Database(String),

    #[error("llm error: {0}")]
    LlmError(String),
}
