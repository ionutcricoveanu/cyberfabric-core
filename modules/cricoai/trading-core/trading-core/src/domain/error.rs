//! Domain-level errors for trading-core

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use trading_core_sdk::TradingCoreError;

/// Domain-level errors
#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("database error: {0}")]
    Database(String),

    #[error("binance api error: {0}")]
    BinanceApi(String),

    #[error("exchange error: {0}")]
    Exchange(String),

    #[error("insufficient balance: {0}")]
    InsufficientBalance(String),

    #[error("invalid order: {0}")]
    InvalidOrder(String),

    #[error("position not found: {0}")]
    PositionNotFound(String),

    #[error("trading is disabled")]
    TradingDisabled,

    #[error("risk limit exceeded: {0}")]
    RiskLimitExceeded(String),

    #[error("invalid configuration: {0}")]
    InvalidConfiguration(String),

    #[error("service error: {0}")]
    ServiceError(String),

    #[error("ml service error: {0}")]
    MlServiceError(String),

    #[error("invalid state: {0}")]
    InvalidState(String),
}

impl From<DomainError> for TradingCoreError {
    fn from(err: DomainError) -> Self {
        match err {
            DomainError::PositionNotFound(msg) => TradingCoreError::NotFound(msg),
            DomainError::Database(msg) => TradingCoreError::Database(msg),
            DomainError::BinanceApi(msg) | DomainError::Exchange(msg) => {
                TradingCoreError::Exchange(msg)
            }
            DomainError::TradingDisabled => {
                TradingCoreError::InvalidState("Trading is disabled".to_string())
            }
            DomainError::InvalidState(msg) => TradingCoreError::InvalidState(msg),
            _ => TradingCoreError::InvalidState(err.to_string()),
        }
    }
}

impl IntoResponse for DomainError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            DomainError::PositionNotFound(msg) => (StatusCode::NOT_FOUND, msg),
            DomainError::Database(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", msg))
            }
            DomainError::BinanceApi(msg) | DomainError::Exchange(msg) => {
                (StatusCode::BAD_GATEWAY, format!("Exchange error: {}", msg))
            }
            DomainError::InsufficientBalance(msg) => {
                (StatusCode::CONFLICT, format!("Insufficient balance: {}", msg))
            }
            DomainError::InvalidOrder(msg) => {
                (StatusCode::BAD_REQUEST, format!("Invalid order: {}", msg))
            }
            DomainError::TradingDisabled => {
                (StatusCode::CONFLICT, "Trading is disabled".to_string())
            }
            DomainError::RiskLimitExceeded(msg) => {
                (StatusCode::CONFLICT, format!("Risk limit exceeded: {}", msg))
            }
            DomainError::InvalidConfiguration(msg) => {
                (StatusCode::BAD_REQUEST, format!("Invalid configuration: {}", msg))
            }
            DomainError::ServiceError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, msg)
            }
            DomainError::MlServiceError(msg) => {
                (StatusCode::BAD_GATEWAY, format!("ML service error: {}", msg))
            }
            DomainError::InvalidState(msg) => {
                (StatusCode::CONFLICT, format!("Invalid state: {}", msg))
            }
        };

        let body = Json(json!({
            "error": error_message,
            "status": status.as_u16(),
        }));

        (status, body).into_response()
    }
}
