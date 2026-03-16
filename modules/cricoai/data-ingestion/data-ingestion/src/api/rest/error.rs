//! Error mapping for REST responses

use data_ingestion_sdk::DataIngestionError;
use http::StatusCode;
use modkit::api::problem::Problem;

/// Map domain errors to HTTP problems
pub fn to_problem(err: DataIngestionError) -> Problem {
    match err {
        DataIngestionError::NotFound(msg) => {
            Problem::new(StatusCode::NOT_FOUND, "Not Found", msg)
        }
        DataIngestionError::Database(msg) => {
            Problem::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database Error",
                format!("Database error: {}", msg),
            )
        }
        DataIngestionError::Source(msg) => {
            Problem::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "External Source Error",
                format!("External source error: {}", msg),
            )
        }
    }
}
