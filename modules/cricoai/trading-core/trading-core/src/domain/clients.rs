//! Integration clients for communication with other modules

use tracing::debug;
use modkit::ClientHub;
use modkit_security::SecurityContext;
use rust_decimal::Decimal;
use std::sync::Arc;

use config_manager_sdk::{ConfigManagerApi, models::PairConfig};
use market_data_sdk::MarketDataApi;
use market_intelligence_sdk::MarketIntelligenceApi;
use ml_service_sdk::MlServiceApi;

use crate::domain::error::DomainError;
use crate::config::TradingCoreConfig;

/// Client integrations for other modules
pub struct ModuleClients {
    client_hub: Arc<ClientHub>,
    config: Arc<TradingCoreConfig>,
}

impl ModuleClients {
    /// Create new module clients
    pub fn new(client_hub: Arc<ClientHub>, config: Arc<TradingCoreConfig>) -> Self {
        Self { client_hub, config }
    }

    fn hub_client<T>(&self) -> Result<Arc<T>, DomainError>
    where
        T: ?Sized + Send + Sync + 'static,
    {
        self.client_hub
            .get::<T>()
            .map_err(|e| DomainError::ServiceError(format!("ClientHub: {e}")))
    }

    /// Fetch market data (klines, ticker) for a symbol
    pub async fn get_market_data(
        &self,
        ctx: &SecurityContext,
        symbol: &str,
    ) -> Result<MarketData, DomainError> {
        let client = self.hub_client::<dyn MarketDataApi>()?;
        debug!("Fetching market data for symbol {}", symbol);

        let ticker = client
            .get_ticker_24h(ctx, symbol)
            .await
            .map_err(|e| DomainError::ServiceError(format!("market-data: {e}")))?;

        Ok(MarketData {
            symbol: ticker.symbol,
            current_price: Decimal::from_f64_retain(ticker.last_price).unwrap_or(Decimal::ZERO),
            bid_price: Decimal::from_f64_retain(ticker.bid_price).unwrap_or(Decimal::ZERO),
            ask_price: Decimal::from_f64_retain(ticker.ask_price).unwrap_or(Decimal::ZERO),
            volume_24h: ticker.volume,
            price_change_pct_24h: ticker.price_change_percent,
        })
    }

    /// Get ML prediction for a symbol
    pub async fn get_ml_prediction(
        &self,
        _ctx: &SecurityContext,
        symbol: &str,
        interval: &str,
    ) -> Result<MlPrediction, DomainError> {
        let client = self.hub_client::<dyn MlServiceApi>()?;
        debug!("Fetching ML prediction for {} interval {}", symbol, interval);

        let prediction = client
            .get_prediction(symbol, interval)
            .await
            .map_err(|e| DomainError::MlServiceError(e.to_string()))?;

        Ok(MlPrediction {
            symbol: prediction.symbol,
            direction: prediction.direction,
            confidence: prediction.confidence,
            predicted_change_pct: prediction.predicted_change_pct,
        })
    }

    /// Get market intelligence decision for a symbol
    pub async fn get_agent_decision(
        &self,
        ctx: &SecurityContext,
        symbol: &str,
    ) -> Result<AgentDecision, DomainError> {
        let client = self.hub_client::<dyn MarketIntelligenceApi>()?;
        debug!("Fetching agent decision for symbol {}", symbol);

        let decision = client
            .get_latest_decision(ctx, symbol)
            .await
            .map_err(|e| DomainError::ServiceError(format!("market-intelligence: {e}")))?;

        Ok(match decision {
            Some(decision) => AgentDecision {
                symbol: decision.symbol,
                action: decision.action,
                confidence: decision.confidence,
                reasoning: decision.reasoning,
            },
            None => AgentDecision {
                symbol: symbol.to_string(),
                action: "NEUTRAL".to_string(),
                confidence: 0.5,
                reasoning: "No decision available".to_string(),
            },
        })
    }

    /// Get trading configuration
    pub async fn get_trading_config(
        &self,
        ctx: &SecurityContext,
    ) -> Result<TradingConfig, DomainError> {
        let market_data = self.hub_client::<dyn MarketDataApi>()?;
        let config_manager = self.hub_client::<dyn ConfigManagerApi>()?;
        debug!("Fetching trading configuration");

        let tickers = market_data
            .get_all_tickers_24h(ctx)
            .await
            .map_err(|e| DomainError::ServiceError(format!("market-data: {e}")))?;

        let mut symbols = Vec::new();
        for ticker in tickers.into_iter().take(50) {
            match config_manager.get_pair_config(ctx, &ticker.symbol).await {
                Ok(PairConfig { excluded, pair, .. }) => {
                    if !excluded {
                        symbols.push(pair);
                    }
                }
                Err(_) => symbols.push(ticker.symbol),
            }
        }

        Ok(TradingConfig {
            symbols,
            buy_signal_threshold: 0.6,
            sell_profit_target: self.config.risk_management.take_profit_pct,
            sell_stop_loss: self.config.risk_management.stop_loss_pct,
        })
    }
}

/// Market data from market-data module
#[derive(Debug, Clone)]
pub struct MarketData {
    pub symbol: String,
    pub current_price: rust_decimal::Decimal,
    pub bid_price: rust_decimal::Decimal,
    pub ask_price: rust_decimal::Decimal,
    pub volume_24h: f64,
    pub price_change_pct_24h: f64,
}

/// ML prediction from ml-service module
#[derive(Debug, Clone)]
pub struct MlPrediction {
    pub symbol: String,
    pub direction: String,  // BUY, SELL, HOLD
    pub confidence: f64,
    pub predicted_change_pct: f64,
}

/// Agent decision from market-intelligence module
#[derive(Debug, Clone)]
pub struct AgentDecision {
    pub symbol: String,
    pub action: String,  // BUY, SELL, HOLD, NEUTRAL
    pub confidence: f64,
    pub reasoning: String,
}

/// Trading configuration from config-manager module
#[derive(Debug, Clone)]
pub struct TradingConfig {
    pub symbols: Vec<String>,
    pub buy_signal_threshold: f64,
    pub sell_profit_target: f64,
    pub sell_stop_loss: f64,
}
