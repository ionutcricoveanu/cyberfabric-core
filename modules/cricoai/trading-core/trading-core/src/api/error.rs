//! API error types for trading-core

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use trading_core_sdk::TradingCoreError;

/// API-level error type
#[derive(Debug)]
pub enum ApiError {
    /// Trading core service error
    TradingCore(TradingCoreError),
    /// Database error
    Database(String),
    /// Invalid request
    InvalidRequest(String),
    /// Resource not found
    NotFound(String),
    /// Unauthorized
    Unauthorized,
    /// Internal server error
    InternalError(String),
}

impl From<TradingCoreError> for ApiError {
    fn from(err: TradingCoreError) -> Self {
        ApiError::TradingCore(err)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiError::TradingCore(TradingCoreError::NotFound(msg)) => {
                (StatusCode::NOT_FOUND, msg)
            }
            ApiError::TradingCore(TradingCoreError::Database(msg)) => {
                (StatusCode::INTERNAL_SERVER_ERROR, msg)
            }
            ApiError::TradingCore(TradingCoreError::Exchange(msg)) => {
                (StatusCode::BAD_GATEWAY, msg)
            }
            ApiError::TradingCore(TradingCoreError::InvalidState(msg)) => {
                (StatusCode::CONFLICT, msg)
            }
            ApiError::Database(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, msg)
            }
            ApiError::InvalidRequest(msg) => {
                (StatusCode::BAD_REQUEST, msg)
            }
            ApiError::NotFound(msg) => {
                (StatusCode::NOT_FOUND, msg)
            }
            ApiError::Unauthorized => {
                (StatusCode::UNAUTHORIZED, "Unauthorized".to_string())
            }
            ApiError::InternalError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, msg)
            }
        };

        let body = Json(json!({
            "error": error_message,
            "status": status.as_u16(),
        }));

        (status, body).into_response()
    }
}
