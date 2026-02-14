//! Error types for the ML service.

#[derive(Debug, thiserror::Error)]
pub enum MlServiceError {
    #[error("connection error: {0}")]
    Connection(String),

    #[error("prediction error: {0}")]
    Prediction(String),

    #[error("model not ready: {0}")]
    ModelNotReady(String),
}
