use crate::config::MarketDataConfig;
use crate::domain::binance_client::BinanceClient;
use crate::domain::collector::KlineCollector;
use crate::domain::entities::klines::*;
use market_data_sdk::errors::MarketDataError;
use market_data_sdk::models::{Kline, KlineInterval, Ticker24h};
use modkit_db::secure::{AccessScope, SecureEntityExt};
use modkit_db::Db;
use modkit_security::SecurityContext;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use std::sync::Arc;

/// Market Data service - holds all dependencies
#[derive(Clone)]
pub struct MarketDataService {
    pub binance_client: Arc<BinanceClient>,
    pub binance_db: Arc<Db>,
    pub klines_db: Arc<Db>,
    pub collector: Arc<KlineCollector>,
    pub config: MarketDataConfig,
}

impl MarketDataService {
    pub fn new(
        binance_db: Arc<Db>,
        klines_db: Arc<Db>,
        config: MarketDataConfig,
    ) -> Result<Arc<Self>, market_data_sdk::MarketDataError> {
        let binance_client = BinanceClient::new(config.binance_url.clone())?;

        let collector = Arc::new(KlineCollector::new(
            binance_client.clone(),
            binance_db.clone(),
            klines_db.clone(),
            config.clone(),
        ));

        Ok(Arc::new(Self {
            binance_client,
            binance_db,
            klines_db,
            collector,
            config,
        }))
    }

    /// Get klines for a symbol and interval from database
    pub async fn get_klines(
        &self,
        _ctx: &SecurityContext,
        symbol: &str,
        interval: KlineInterval,
        limit: Option<u32>,
    ) -> Result<Vec<Kline>, MarketDataError> {
        let conn = self.klines_db.conn().map_err(|e| {
            MarketDataError::DatabaseError(format!("Failed to get database connection: {e}"))
        })?;

        let scope = AccessScope::default();
        let limit = limit.unwrap_or(100).min(1000) as u64;

        // Query the appropriate interval-specific table
        let klines = match interval {
            KlineInterval::OneMinute => {
                let models = klines_1m::Entity::find()
                    .filter(klines_1m::Column::Symbol.eq(symbol))
                    .order_by_desc(klines_1m::Column::OpenTime)
                    .limit(limit)
                    .secure()
                    .scope_with(&scope)
                    .all(&conn)
                    .await
                    .map_err(|e| MarketDataError::DatabaseError(format!("Query failed: {e}")))?;

                models.into_iter().map(Kline::from).collect()
            }
            KlineInterval::FiveMinutes => {
                let models = klines_5m::Entity::find()
                    .filter(klines_5m::Column::Symbol.eq(symbol))
                    .order_by_desc(klines_5m::Column::OpenTime)
                    .limit(limit)
                    .secure()
                    .scope_with(&scope)
                    .all(&conn)
                    .await
                    .map_err(|e| MarketDataError::DatabaseError(format!("Query failed: {e}")))?;

                models.into_iter().map(Kline::from).collect()
            }
            KlineInterval::FifteenMinutes => {
                let models = klines_15m::Entity::find()
                    .filter(klines_15m::Column::Symbol.eq(symbol))
                    .order_by_desc(klines_15m::Column::OpenTime)
                    .limit(limit)
                    .secure()
                    .scope_with(&scope)
                    .all(&conn)
                    .await
                    .map_err(|e| MarketDataError::DatabaseError(format!("Query failed: {e}")))?;

                models.into_iter().map(Kline::from).collect()
            }
            KlineInterval::OneHour => {
                let models = klines_1h::Entity::find()
                    .filter(klines_1h::Column::Symbol.eq(symbol))
                    .order_by_desc(klines_1h::Column::OpenTime)
                    .limit(limit)
                    .secure()
                    .scope_with(&scope)
                    .all(&conn)
                    .await
                    .map_err(|e| MarketDataError::DatabaseError(format!("Query failed: {e}")))?;

                models.into_iter().map(Kline::from).collect()
            }
            KlineInterval::FourHours => {
                let models = klines_4h::Entity::find()
                    .filter(klines_4h::Column::Symbol.eq(symbol))
                    .order_by_desc(klines_4h::Column::OpenTime)
                    .limit(limit)
                    .secure()
                    .scope_with(&scope)
                    .all(&conn)
                    .await
                    .map_err(|e| MarketDataError::DatabaseError(format!("Query failed: {e}")))?;

                models.into_iter().map(Kline::from).collect()
            }
        };

        Ok(klines)
    }

    /// Get 24h ticker statistics for a symbol from Binance API
    pub async fn get_ticker_24h(
        &self,
        _ctx: &SecurityContext,
        symbol: &str,
    ) -> Result<Ticker24h, MarketDataError> {
        self.binance_client.fetch_ticker_24h(symbol).await
    }

    /// Get 24h ticker statistics for all symbols from Binance API
    pub async fn get_all_tickers_24h(
        &self,
        _ctx: &SecurityContext,
    ) -> Result<Vec<Ticker24h>, MarketDataError> {
        self.binance_client.fetch_all_tickers_24h().await
    }

    /// Get the latest kline for a symbol and interval
    pub async fn get_latest_kline(
        &self,
        _ctx: &SecurityContext,
        symbol: &str,
        interval: KlineInterval,
    ) -> Result<Option<Kline>, MarketDataError> {
        let conn = self.klines_db.conn().map_err(|e| {
            MarketDataError::DatabaseError(format!("Failed to get database connection: {e}"))
        })?;

        let scope = AccessScope::default();

        let kline = match interval {
            KlineInterval::OneMinute => {
                klines_1m::Entity::find()
                    .filter(klines_1m::Column::Symbol.eq(symbol))
                    .order_by_desc(klines_1m::Column::OpenTime)
                    .limit(1)
                    .secure()
                    .scope_with(&scope)
                    .one(&conn)
                    .await
                    .map_err(|e| MarketDataError::DatabaseError(format!("Query failed: {e}")))?
                    .map(Kline::from)
            }
            KlineInterval::FiveMinutes => {
                klines_5m::Entity::find()
                    .filter(klines_5m::Column::Symbol.eq(symbol))
                    .order_by_desc(klines_5m::Column::OpenTime)
                    .limit(1)
                    .secure()
                    .scope_with(&scope)
                    .one(&conn)
                    .await
                    .map_err(|e| MarketDataError::DatabaseError(format!("Query failed: {e}")))?
                    .map(Kline::from)
            }
            KlineInterval::FifteenMinutes => {
                klines_15m::Entity::find()
                    .filter(klines_15m::Column::Symbol.eq(symbol))
                    .order_by_desc(klines_15m::Column::OpenTime)
                    .limit(1)
                    .secure()
                    .scope_with(&scope)
                    .one(&conn)
                    .await
                    .map_err(|e| MarketDataError::DatabaseError(format!("Query failed: {e}")))?
                    .map(Kline::from)
            }
            KlineInterval::OneHour => {
                klines_1h::Entity::find()
                    .filter(klines_1h::Column::Symbol.eq(symbol))
                    .order_by_desc(klines_1h::Column::OpenTime)
                    .limit(1)
                    .secure()
                    .scope_with(&scope)
                    .one(&conn)
                    .await
                    .map_err(|e| MarketDataError::DatabaseError(format!("Query failed: {e}")))?
                    .map(Kline::from)
            }
            KlineInterval::FourHours => {
                klines_4h::Entity::find()
                    .filter(klines_4h::Column::Symbol.eq(symbol))
                    .order_by_desc(klines_4h::Column::OpenTime)
                    .limit(1)
                    .secure()
                    .scope_with(&scope)
                    .one(&conn)
                    .await
                    .map_err(|e| MarketDataError::DatabaseError(format!("Query failed: {e}")))?
                    .map(Kline::from)
            }
        };

        Ok(kline)
    }

    /// Check if klines exist for a symbol and interval
    pub async fn has_klines(
        &self,
        _ctx: &SecurityContext,
        symbol: &str,
        interval: KlineInterval,
    ) -> Result<bool, MarketDataError> {
        let conn = self.klines_db.conn().map_err(|e| {
            MarketDataError::DatabaseError(format!("Failed to get database connection: {e}"))
        })?;

        let scope = AccessScope::default();

        let count: u64 = match interval {
            KlineInterval::OneMinute => {
                klines_1m::Entity::find()
                    .filter(klines_1m::Column::Symbol.eq(symbol))
                    .limit(1)
                    .secure()
                    .scope_with(&scope)
                    .count(&conn)
                    .await
                    .map_err(|e| MarketDataError::DatabaseError(format!("Query failed: {e}")))?
            }
            KlineInterval::FiveMinutes => {
                klines_5m::Entity::find()
                    .filter(klines_5m::Column::Symbol.eq(symbol))
                    .limit(1)
                    .secure()
                    .scope_with(&scope)
                    .count(&conn)
                    .await
                    .map_err(|e| MarketDataError::DatabaseError(format!("Query failed: {e}")))?
            }
            KlineInterval::FifteenMinutes => {
                klines_15m::Entity::find()
                    .filter(klines_15m::Column::Symbol.eq(symbol))
                    .limit(1)
                    .secure()
                    .scope_with(&scope)
                    .count(&conn)
                    .await
                    .map_err(|e| MarketDataError::DatabaseError(format!("Query failed: {e}")))?
            }
            KlineInterval::OneHour => {
                klines_1h::Entity::find()
                    .filter(klines_1h::Column::Symbol.eq(symbol))
                    .limit(1)
                    .secure()
                    .scope_with(&scope)
                    .count(&conn)
                    .await
                    .map_err(|e| MarketDataError::DatabaseError(format!("Query failed: {e}")))?
            }
            KlineInterval::FourHours => {
                klines_4h::Entity::find()
                    .filter(klines_4h::Column::Symbol.eq(symbol))
                    .limit(1)
                    .secure()
                    .scope_with(&scope)
                    .count(&conn)
                    .await
                    .map_err(|e| MarketDataError::DatabaseError(format!("Query failed: {e}")))?
            }
        };

        Ok(count > 0)
    }
}
