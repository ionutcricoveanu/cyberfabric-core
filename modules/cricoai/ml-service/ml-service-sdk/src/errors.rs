//! Error types for the ML service.

#[derive(Debug, thiserror::Error)]
pub enum MlServiceError {
    #[error("connection error: {0}")]
    ConnectionError(String),

    #[error("service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("prediction failed: {0}")]
    PredictionFailed(String),

    #[error("indicators failed: {0}")]
    IndicatorsFailed(String),

    #[error("market regime failed: {0}")]
    RegimeFailed(String),

    #[error("model not ready: {0}")]
    ModelNotReady(String),

    #[error("invalid input: {0}")]
    InvalidInput(String),
}
