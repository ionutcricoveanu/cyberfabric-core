//! Object-safe client trait for the ML service.

use async_trait::async_trait;

use crate::errors::MlServiceError;
use crate::models::{MarketRegime, Prediction, TechnicalIndicators};

#[async_trait]
pub trait MlServiceApi: Send + Sync {
    /// Get ML prediction for a symbol and interval.
    async fn get_prediction(
        &self,
        symbol: &str,
        interval: &str,
    ) -> Result<Prediction, MlServiceError>;

    /// Get technical indicators for a symbol.
    async fn get_technical_indicators(
        &self,
        symbol: &str,
    ) -> Result<TechnicalIndicators, MlServiceError>;

    /// Get current market regime for a symbol.
    async fn get_market_regime(
        &self,
        symbol: &str,
    ) -> Result<MarketRegime, MlServiceError>;
}
