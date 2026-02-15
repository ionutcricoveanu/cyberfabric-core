use thiserror::Error;

#[derive(Debug, Error)]
pub enum AgentTradeImpactError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Database error: {0}")]
    Database(String),
}
