//! gRPC client for ML Service
//!
//! This module provides a gRPC client that connects to the Python ML service
//! and implements the `MlServiceApi` trait.

use async_trait::async_trait;
use tonic::transport::Channel;
use tracing::{debug, error, instrument};

use crate::client::MlServiceApi;
use crate::errors::MlServiceError;
use crate::models::{MarketRegime, Prediction, TechnicalIndicators};
use crate::ml_service::ml_service_client::MlServiceClient as TonicMlServiceClient;
use crate::ml_service::{
    GetPredictionRequest, GetIndicatorsRequest, GetMarketRegimeRequest, HealthCheckRequest,
};

/// ML Service gRPC client
pub struct MlServiceClient {
    client: TonicMlServiceClient<Channel>,
}

impl MlServiceClient {
    /// Create a new ML Service gRPC client
    pub async fn connect(addr: String) -> Result<Self, MlServiceError> {
        debug!("Connecting to ML Service at {}", addr);

        let channel = Channel::from_shared(addr)
            .map_err(|e| MlServiceError::ConnectionError(format!("Invalid URL: {}", e)))?
            .connect()
            .await
            .map_err(|e| {
                MlServiceError::ConnectionError(format!("Failed to connect: {}", e))
            })?;

        debug!("Connected to ML Service");

        Ok(Self {
            client: TonicMlServiceClient::new(channel),
        })
    }

    /// Perform a health check on the ML Service
    #[instrument(skip(self))]
    pub async fn health_check(&mut self) -> Result<bool, MlServiceError> {
        debug!("Performing health check on ML Service");

        let request = HealthCheckRequest {
            service: "ml-service".to_string(),
        };

        match self.client.health(request).await {
            Ok(response) => {
                let status = response.into_inner().status();
                let is_serving = status as i32 == 1; // SERVING = 1
                debug!("Health check result: {}", is_serving);
                Ok(is_serving)
            }
            Err(e) => {
                error!("Health check failed: {}", e);
                Err(MlServiceError::ServiceUnavailable(format!(
                    "Health check failed: {}",
                    e
                )))
            }
        }
    }
}

#[async_trait]
impl MlServiceApi for MlServiceClient {
    #[instrument(skip(self))]
    async fn get_prediction(
        &self,
        symbol: &str,
        interval: &str,
    ) -> Result<Prediction, MlServiceError> {
        debug!("Requesting prediction for {} on {}", symbol, interval);

        let mut client = self.client.clone();
        let request = GetPredictionRequest {
            symbol: symbol.to_string(),
            interval: interval.to_string(),
        };

        match client.get_prediction(request).await {
            Ok(response) => {
                let resp = response.into_inner();
                debug!(
                    "Got prediction for {}: {} (confidence: {:.2}%)",
                    symbol,
                    resp.direction,
                    resp.confidence * 100.0
                );

                Ok(Prediction {
                    symbol: resp.symbol,
                    interval: resp.interval,
                    direction: resp.direction,
                    confidence: resp.confidence,
                    predicted_change_pct: resp.predicted_change_pct,
                })
            }
            Err(e) => {
                error!("Prediction request failed for {}: {}", symbol, e);
                Err(MlServiceError::PredictionFailed(format!(
                    "Failed to get prediction: {}",
                    e
                )))
            }
        }
    }

    #[instrument(skip(self))]
    async fn get_technical_indicators(
        &self,
        symbol: &str,
    ) -> Result<TechnicalIndicators, MlServiceError> {
        debug!("Requesting technical indicators for {}", symbol);

        let mut client = self.client.clone();
        let request = GetIndicatorsRequest {
            symbol: symbol.to_string(),
        };

        match client.get_technical_indicators(request).await {
            Ok(response) => {
                let resp = response.into_inner();
                debug!("Got technical indicators for {}: RSI={:.2}", symbol, resp.rsi);

                Ok(TechnicalIndicators {
                    symbol: resp.symbol,
                    rsi: resp.rsi,
                    macd: resp.macd,
                    macd_signal: resp.macd_signal,
                    bollinger_upper: resp.bollinger_upper,
                    bollinger_lower: resp.bollinger_lower,
                    ema_12: resp.ema_12,
                    ema_26: resp.ema_26,
                })
            }
            Err(e) => {
                error!("Technical indicators request failed for {}: {}", symbol, e);
                Err(MlServiceError::IndicatorsFailed(format!(
                    "Failed to get indicators: {}",
                    e
                )))
            }
        }
    }

    #[instrument(skip(self))]
    async fn get_market_regime(
        &self,
        symbol: &str,
    ) -> Result<MarketRegime, MlServiceError> {
        debug!("Requesting market regime for {}", symbol);

        let mut client = self.client.clone();
        let request = GetMarketRegimeRequest {
            symbol: symbol.to_string(),
        };

        match client.get_market_regime(request).await {
            Ok(response) => {
                let resp = response.into_inner();
                debug!(
                    "Got market regime for {}: {} (confidence: {:.2}%)",
                    symbol,
                    resp.regime,
                    resp.confidence * 100.0
                );

                Ok(MarketRegime {
                    symbol: resp.symbol,
                    regime: resp.regime,
                    confidence: resp.confidence,
                })
            }
            Err(e) => {
                error!("Market regime request failed for {}: {}", symbol, e);
                Err(MlServiceError::RegimeFailed(format!(
                    "Failed to get regime: {}",
                    e
                )))
            }
        }
    }
}
