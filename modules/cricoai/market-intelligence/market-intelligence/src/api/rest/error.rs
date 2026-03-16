//! Error mapping for REST responses

use market_intelligence_sdk::MarketIntelligenceError;
use http::StatusCode;
use modkit::api::problem::Problem;

/// Map domain errors to HTTP problems
pub fn to_problem(err: MarketIntelligenceError) -> Problem {
    match err {
        MarketIntelligenceError::NotFound(msg) => {
            Problem::new(StatusCode::NOT_FOUND, "Not Found", msg)
        }
        MarketIntelligenceError::Database(msg) => {
            Problem::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database Error",
                format!("Database error: {}", msg),
            )
        }
        MarketIntelligenceError::LlmError(msg) => {
            Problem::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "LLM Error",
                format!("LLM error: {}", msg),
            )
        }
    }
}
