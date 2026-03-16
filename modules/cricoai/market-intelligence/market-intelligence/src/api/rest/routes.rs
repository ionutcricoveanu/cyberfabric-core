//! REST route registration

use axum::{
    routing::get,
    Router,
};

use super::handlers::*;

/// Register REST routes for the market-intelligence module
pub fn routes() -> Router {
    Router::new()
        // GET /market-intelligence/v1/decision/{symbol}
        .route("/v1/decision/{symbol}", get(get_latest_decision))
        // GET /market-intelligence/v1/status
        .route("/v1/status", get(get_status))
}
