use crate::config::MarketDataConfig;
use crate::domain::binance_client::BinanceClient;
use crate::domain::entities::trade_pairs;
use market_data_sdk::MarketDataError;
use modkit_db::secure::{AccessScope, SecureEntityExt, SecureInsertExt};
use modkit_db::Db;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::{interval, sleep};
use tracing::{error, info};

/// Kline collector - polls Binance and stores klines
pub struct KlineCollector {
    binance_client: Arc<BinanceClient>,
    binance_db: Arc<Db>,
    _klines_db: Arc<Db>,
    config: MarketDataConfig,
}

impl KlineCollector {
    pub fn new(
        binance_client: Arc<BinanceClient>,
        binance_db: Arc<Db>,
        klines_db: Arc<Db>,
        config: MarketDataConfig,
    ) -> Self {
        Self {
            binance_client,
            binance_db,
            _klines_db: klines_db,
            config,
        }
    }

    /// Main collection loop - runs until cancelled
    pub async fn run(&self, cancel_token: tokio_util::sync::CancellationToken) {
        info!("Starting kline collector");

        // Test connectivity first
        if let Err(e) = self.binance_client.test_connectivity().await {
            error!("Failed to connect to Binance API: {}. Retrying in 30s...", e);
            sleep(Duration::from_secs(30)).await;
        }

        let mut tick = interval(Duration::from_secs(self.config.collection_interval_secs));

        loop {
            tokio::select! {
                _ = cancel_token.cancelled() => {
                    info!("Kline collector received cancellation signal");
                    break;
                }
                _ = tick.tick() => {
                    if let Err(e) = self.collect_all_klines().await {
                        error!("Error during kline collection cycle: {}", e);
                    }
                }
            }
        }

        info!("Kline collector stopped");
    }

    /// Collect klines for all symbols and all configured intervals
    async fn collect_all_klines(&self) -> Result<(), MarketDataError> {
        // Fetch active trading pairs from database
        let symbols = self.fetch_active_symbols().await?;

        if symbols.is_empty() {
            error!("No active trading pairs found in database");
            return Ok(());
        }

        info!("Collecting klines for {} symbols", symbols.len());

        let intervals = self.config.parse_intervals();

        // Collect klines for each symbol and interval
        for symbol in &symbols {
            for interval in &intervals {
                match self.collect_klines_for_symbol(symbol, interval).await {
                    Ok(count) => {
                        if count > 0 {
                            info!("Fetched {} klines for {} ({})", count, symbol, interval);
                        }
                    }
                    Err(e) => {
                        error!("Failed to collect klines for {} ({}): {}", symbol, interval, e);
                        // Continue with next symbol/interval instead of failing entire cycle
                    }
                }

                // Small delay between requests to avoid rate limiting
                sleep(Duration::from_millis(100)).await;
            }
        }

        Ok(())
    }

    /// Fetch active trading symbols from the Binance database
    async fn fetch_active_symbols(&self) -> Result<Vec<String>, MarketDataError> {
        // Get database connection
        let conn = self.binance_db.conn().map_err(|e| {
            MarketDataError::DatabaseError(format!("Failed to get database connection: {e}"))
        })?;

        // Use unrestricted entity with default scope
        let scope = AccessScope::default();

        // Query active trading pairs using secure entity pattern
        let pairs = trade_pairs::Entity::find()
            .filter(trade_pairs::Column::Active.eq(true))
            .order_by_asc(trade_pairs::Column::Pair)
            .secure()
            .scope_with(&scope)
            .all(&conn)
            .await
            .map_err(|e| {
                MarketDataError::DatabaseError(format!("Failed to fetch symbols: {e}"))
            })?;

        let symbols: Vec<String> = pairs.into_iter().map(|p| p.pair).collect();
        Ok(symbols)
    }

    /// Collect klines for a specific symbol and interval
    async fn collect_klines_for_symbol(
        &self,
        symbol: &str,
        interval: &str,
    ) -> Result<usize, MarketDataError> {
        // Fetch klines from Binance
        let klines = self
            .binance_client
            .fetch_klines(symbol, interval, self.config.limit)
            .await?;

        if klines.is_empty() {
            return Ok(0);
        }

        // Store in database using interval-specific table
        self.store_klines(&klines, interval).await?;

        Ok(klines.len())
    }

    /// Store klines in the appropriate interval-specific table
    async fn store_klines(
        &self,
        klines: &[market_data_sdk::models::Kline],
        interval: &str,
    ) -> Result<(), MarketDataError> {
        use crate::domain::entities::klines::*;
        use modkit_db::secure::AccessScope;
        use sea_orm::EntityTrait;

        let conn = self._klines_db.conn().map_err(|e| {
            MarketDataError::DatabaseError(format!("Failed to get klines DB connection: {e}"))
        })?;

        let scope = AccessScope::default();

        // Convert all klines to active models and batch insert
        // Note: We ignore duplicate key errors as we may re-collect the same klines
        match interval {
            "1m" => {
                for kline in klines {
                    let active_model: klines_1m::ActiveModel = kline.into();
                    // Insert using secure entity pattern
                    let _ = klines_1m::Entity::insert(active_model)
                        .secure()
                        .scope_with(&scope)
                        .map_err(|e| MarketDataError::DatabaseError(format!("Scope error: {e}")))?
                        .exec(&conn)
                        .await;
                    // Ignore errors (duplicate keys expected)
                }
            }
            "5m" => {
                for kline in klines {
                    let active_model: klines_5m::ActiveModel = kline.into();
                    let _ = klines_5m::Entity::insert(active_model)
                        .secure()
                        .scope_with(&scope)
                        .map_err(|e| MarketDataError::DatabaseError(format!("Scope error: {e}")))?
                        .exec(&conn)
                        .await;
                }
            }
            "15m" => {
                for kline in klines {
                    let active_model: klines_15m::ActiveModel = kline.into();
                    let _ = klines_15m::Entity::insert(active_model)
                        .secure()
                        .scope_with(&scope)
                        .map_err(|e| MarketDataError::DatabaseError(format!("Scope error: {e}")))?
                        .exec(&conn)
                        .await;
                }
            }
            "1h" => {
                for kline in klines {
                    let active_model: klines_1h::ActiveModel = kline.into();
                    let _ = klines_1h::Entity::insert(active_model)
                        .secure()
                        .scope_with(&scope)
                        .map_err(|e| MarketDataError::DatabaseError(format!("Scope error: {e}")))?
                        .exec(&conn)
                        .await;
                }
            }
            "4h" => {
                for kline in klines {
                    let active_model: klines_4h::ActiveModel = kline.into();
                    let _ = klines_4h::Entity::insert(active_model)
                        .secure()
                        .scope_with(&scope)
                        .map_err(|e| MarketDataError::DatabaseError(format!("Scope error: {e}")))?
                        .exec(&conn)
                        .await;
                }
            }
            _ => {
                return Err(MarketDataError::InvalidInterval(format!(
                    "Unsupported interval: {}",
                    interval
                )))
            }
        }

        Ok(())
    }
}
