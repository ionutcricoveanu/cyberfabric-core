use http::StatusCode;
use market_data_sdk::errors::MarketDataError;
use modkit_errors::Problem;

/// Convert MarketDataError to HTTP Problem
pub fn to_problem(err: MarketDataError) -> Problem {
    match err {
        MarketDataError::InvalidSymbol(msg) => {
            Problem::new(StatusCode::BAD_REQUEST, "Invalid Symbol", msg)
        }
        MarketDataError::InvalidInterval(msg) => {
            Problem::new(StatusCode::BAD_REQUEST, "Invalid Interval", msg)
        }
        MarketDataError::DatabaseError(msg) => {
            Problem::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database Error",
                format!("Database error: {}", msg),
            )
        }
        MarketDataError::BinanceApiError(msg) => {
            Problem::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "Binance API Error",
                format!("Binance API error: {}", msg),
            )
        }
        MarketDataError::RateLimitExceeded(msg) => {
            Problem::new(
                StatusCode::TOO_MANY_REQUESTS,
                "Rate Limit Exceeded",
                format!("Rate limit exceeded: {}", msg),
            )
        }
        MarketDataError::NetworkError(msg) => {
            Problem::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "Network Error",
                format!("Network error: {}", msg),
            )
        }
        MarketDataError::InternalError(msg) => {
            Problem::new(StatusCode::INTERNAL_SERVER_ERROR, "Internal Error", msg)
        }
    }
}
