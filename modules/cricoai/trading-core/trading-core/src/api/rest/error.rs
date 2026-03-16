//! REST error mapping for trading-core

use http::StatusCode;
use modkit_errors::Problem;
use trading_core_sdk::TradingCoreError;

/// Convert TradingCoreError to HTTP Problem
pub fn to_problem(err: TradingCoreError) -> Problem {
    match err {
        TradingCoreError::NotFound(msg) => {
            Problem::new(StatusCode::NOT_FOUND, "Not Found", msg)
        }
        TradingCoreError::Database(msg) => {
            Problem::new(StatusCode::INTERNAL_SERVER_ERROR, "Database Error", msg)
        }
        TradingCoreError::Exchange(msg) => {
            Problem::new(StatusCode::BAD_GATEWAY, "Exchange Error", msg)
        }
        TradingCoreError::InvalidState(msg) => {
            Problem::new(StatusCode::CONFLICT, "Invalid State", msg)
        }
    }
}
