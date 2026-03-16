//! REST route registration

use axum::{
    routing::get,
    Router,
};

use super::handlers::*;

/// Register REST routes for the data-ingestion module
pub fn routes() -> Router {
    Router::new()
        // GET /data-ingestion/v1/sentiment/{symbol}
        .route("/v1/sentiment/{symbol}", get(get_latest_sentiment))
        // GET /data-ingestion/v1/status
        .route("/v1/status", get(get_status))
}
