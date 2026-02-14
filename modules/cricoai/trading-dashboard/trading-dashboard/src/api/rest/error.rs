use modkit::api::problem::{self, Problem};
use trading_dashboard_sdk::errors::TradingDashboardError;

/// Convert a `TradingDashboardError` into an RFC 9457 Problem.
pub fn to_problem(e: TradingDashboardError) -> Problem {
    match e {
        TradingDashboardError::NotFound(msg) => problem::not_found(msg),
        TradingDashboardError::Database(msg) => {
            tracing::error!(error = %msg, "Database error occurred");
            problem::internal_error("An internal database error occurred")
        }
        TradingDashboardError::InvalidQuery(msg) => problem::bad_request(msg),
    }
}
