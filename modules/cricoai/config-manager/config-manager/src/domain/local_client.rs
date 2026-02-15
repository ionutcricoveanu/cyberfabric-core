use std::sync::Arc;

use async_trait::async_trait;
use modkit_security::SecurityContext;

use config_manager_sdk::errors::ConfigManagerError;
use config_manager_sdk::models::PairConfig;
use config_manager_sdk::client::ConfigManagerApi;

use super::service::ConfigManagerService;

/// Local client adapter that implements the SDK API trait for in-process communication.
pub struct ConfigManagerLocalClient {
    service: Arc<ConfigManagerService>,
}

impl ConfigManagerLocalClient {
    pub fn new(service: Arc<ConfigManagerService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl ConfigManagerApi for ConfigManagerLocalClient {
    async fn get_pair_config(
        &self,
        ctx: &SecurityContext,
        symbol: &str,
    ) -> Result<PairConfig, ConfigManagerError> {
        let pairs = self.service.get_trade_pairs(ctx).await.map_err(|e| {
            ConfigManagerError::Database(e.to_string())
        })?;

        pairs
            .pairs
            .into_iter()
            .find(|p| p.pair == symbol)
            .map(|p| PairConfig {
                pair: p.pair,
                change_24h: p.change_24h,
                excluded: p.excluded,
                highest_percent: p.highest_percent,
                highest_price: p.highest_price,
                current_price: p.current_price,
                lowest_price: p.lowest_price,
                lowest_percent: p.lowest_percent,
                open_orders: p.open_orders,
                last_purchase_time: p.last_purchase_time,
                last_sell_time: p.last_sell_time,
                est_profit_percent: p.est_profit_percent,
            })
            .ok_or_else(|| ConfigManagerError::NotFound(format!("Pair {symbol} not found")))
    }
}
