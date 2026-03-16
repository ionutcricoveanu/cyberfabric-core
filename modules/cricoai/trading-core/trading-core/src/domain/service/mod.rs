//! Business logic service layer

use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::Mutex;
use sea_orm::{ColumnTrait, ConnectionTrait, Database, DbErr, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Statement};
use tracing::{info, error};
use modkit_security::SecurityContext;
use trading_core_sdk::{BotStatus, TradingCoreError};

use crate::config::TradingCoreConfig;
use crate::domain::error::DomainError;
use crate::api::rest::dto::*;
use crate::domain::entities::{buy_orders, sell_orders};

/// Main trading core service
pub struct TradingCoreService {
    #[allow(dead_code)]
    config: Arc<TradingCoreConfig>,
    db_binance: Arc<Mutex<Option<sea_orm::DbConn>>>,
    #[allow(dead_code)]
    db_klines: Arc<Mutex<Option<sea_orm::DbConn>>>,
    #[allow(dead_code)]
    db_model_data: Arc<Mutex<Option<sea_orm::DbConn>>>,
    trading_paused: Arc<RwLock<bool>>,
}

impl TradingCoreService {
    /// Create a new trading core service
    pub async fn new(config: TradingCoreConfig) -> Result<Self, DomainError> {
        let config = Arc::new(config);

        // Initialize database connections
        let db_binance = match Database::connect(&config.binance_dsn).await {
            Ok(db) => Some(db),
            Err(e) => {
                error!("Failed to connect to Binance database: {}", e);
                return Err(DomainError::Database(e.to_string()));
            }
        };

        let db_klines = match Database::connect(&config.klines_dsn).await {
            Ok(db) => Some(db),
            Err(e) => {
                error!("Failed to connect to Binance Klines database: {}", e);
                return Err(DomainError::Database(e.to_string()));
            }
        };

        let db_model_data = match Database::connect(&config.model_data_dsn).await {
            Ok(db) => Some(db),
            Err(e) => {
                error!("Failed to connect to Model Data database: {}", e);
                return Err(DomainError::Database(e.to_string()));
            }
        };

        info!("Trading Core Service initialized with {} testnet={}", 
              if config.testnet { "testnet" } else { "production" },
              config.testnet);

        Ok(Self {
            config,
            db_binance: Arc::new(Mutex::new(db_binance)),
            db_klines: Arc::new(Mutex::new(db_klines)),
            db_model_data: Arc::new(Mutex::new(db_model_data)),
            trading_paused: Arc::new(RwLock::new(false)),
        })
    }

    async fn binance_conn(&self) -> Result<sea_orm::DbConn, TradingCoreError> {
        let guard = self.db_binance.lock().await;
        guard
            .as_ref()
            .cloned()
            .ok_or_else(|| TradingCoreError::Database("Binance database not configured".to_string()))
    }

    fn row_get<T: sea_orm::TryGetable>(
        row: &sea_orm::QueryResult,
        col: &str,
    ) -> Result<T, DbErr> {
        row.try_get("", col)
    }

    /// Get current bot status
    pub async fn get_status(&self, _ctx: &SecurityContext) -> Result<BotStatus, TradingCoreError> {
        let conn = self.binance_conn().await?;

        let open_positions = buy_orders::Entity::find()
            .filter(buy_orders::Column::Status.eq("NEW"))
            .count(&conn)
            .await
            .map_err(|e| TradingCoreError::Database(e.to_string()))? as i64;

        let last_trade_time = sell_orders::Entity::find()
            .order_by_desc(sell_orders::Column::TransactTime)
            .one(&conn)
            .await
            .map_err(|e| TradingCoreError::Database(e.to_string()))?
            .and_then(|row| row.transact_time);

        Ok(BotStatus {
            running: !*self.trading_paused.read(),
            paused: *self.trading_paused.read(),
            open_positions,
            last_trade_time,
        })
    }

    /// Pause trading
    pub async fn pause(&self, _ctx: &SecurityContext) -> Result<(), TradingCoreError> {
        *self.trading_paused.write() = true;
        info!("Trading paused");
        Ok(())
    }

    /// Resume trading
    pub async fn resume(&self, _ctx: &SecurityContext) -> Result<(), TradingCoreError> {
        *self.trading_paused.write() = false;
        info!("Trading resumed");
        Ok(())
    }

    /// Get open positions
    pub async fn get_positions(
        &self,
        _ctx: &SecurityContext,
    ) -> Result<Vec<PositionInfo>, TradingCoreError> {
        let conn = self.binance_conn().await?;

        let orders = buy_orders::Entity::find()
            .filter(buy_orders::Column::Status.eq("NEW"))
            .order_by_desc(buy_orders::Column::TransactTime)
            .all(&conn)
            .await
            .map_err(|e| TradingCoreError::Database(e.to_string()))?;

        let positions = orders
            .into_iter()
            .map(|order| {
                let entry_price = order.order_price.or(order.price).unwrap_or(0.0);
                let quantity = order.qty.or(order.executed_qty).or(order.orig_qty).unwrap_or(0.0);
                let entry_time = order
                    .transact_time
                    .map(|dt| dt)
                    .unwrap_or_else(|| chrono::Utc::now().naive_utc());

                PositionInfo {
                    symbol: order.symbol.unwrap_or_else(|| "UNKNOWN".to_string()),
                    quantity,
                    entry_price,
                    current_price: entry_price,
                    pnl: 0.0,
                    pnl_pct: 0.0,
                    entry_time,
                    stop_loss: order.stop_price,
                    take_profit: None,
                }
            })
            .collect();

        Ok(positions)
    }

    /// Get open position for a symbol
    pub async fn get_position(
        &self,
        _ctx: &SecurityContext,
        _symbol: &str,
    ) -> Result<Option<PositionInfo>, TradingCoreError> {
        let conn = self.binance_conn().await?;

        let order = buy_orders::Entity::find()
            .filter(buy_orders::Column::Status.eq("NEW"))
            .filter(buy_orders::Column::Symbol.eq(_symbol))
            .order_by_desc(buy_orders::Column::TransactTime)
            .one(&conn)
            .await
            .map_err(|e| TradingCoreError::Database(e.to_string()))?;

        let position = order.map(|order| {
            let entry_price = order.order_price.or(order.price).unwrap_or(0.0);
            let quantity = order.qty.or(order.executed_qty).or(order.orig_qty).unwrap_or(0.0);
            let entry_time = order
                .transact_time
                .map(|dt| dt)
                .unwrap_or_else(|| chrono::Utc::now().naive_utc());

            PositionInfo {
                symbol: order.symbol.unwrap_or_else(|| "UNKNOWN".to_string()),
                quantity,
                entry_price,
                current_price: entry_price,
                pnl: 0.0,
                pnl_pct: 0.0,
                entry_time,
                stop_loss: order.stop_price,
                take_profit: None,
            }
        });

        Ok(position)
    }

    /// Get order history
    pub async fn get_order_history(
        &self,
        _ctx: &SecurityContext,
    ) -> Result<Vec<OrderHistoryEntry>, TradingCoreError> {
        let conn = self.binance_conn().await?;

        let buy_rows = buy_orders::Entity::find()
            .order_by_desc(buy_orders::Column::TransactTime)
            .limit(100)
            .all(&conn)
            .await
            .map_err(|e| TradingCoreError::Database(e.to_string()))?;

        let sell_rows = sell_orders::Entity::find()
            .order_by_desc(sell_orders::Column::TransactTime)
            .limit(100)
            .all(&conn)
            .await
            .map_err(|e| TradingCoreError::Database(e.to_string()))?;

        let mut entries: Vec<OrderHistoryEntry> = buy_rows
            .into_iter()
            .map(|order| {
                let price = order.order_price.or(order.price).unwrap_or(0.0);
                let quantity = order.qty.or(order.executed_qty).or(order.orig_qty).unwrap_or(0.0);
                let timestamp = order
                    .transact_time
                    .map(|dt| dt)
                    .unwrap_or_else(|| chrono::Utc::now().naive_utc());

                OrderHistoryEntry {
                    id: order.id as i64,
                    symbol: order.symbol.unwrap_or_else(|| "UNKNOWN".to_string()),
                    side: order.side.unwrap_or_else(|| "BUY".to_string()),
                    quantity,
                    price,
                    total_usdt: price * quantity,
                    timestamp,
                    status: order.status.unwrap_or_else(|| "UNKNOWN".to_string()),
                }
            })
            .collect();

        entries.extend(sell_rows.into_iter().map(|order| {
            let price = order.order_price.or(order.price).unwrap_or(0.0);
            let quantity = order.qty.or(order.executed_qty).or(order.orig_qty).unwrap_or(0.0);
            let timestamp = order
                .transact_time
                .map(|dt| dt)
                .unwrap_or_else(|| chrono::Utc::now().naive_utc());

            OrderHistoryEntry {
                id: order.id as i64,
                symbol: order.symbol.unwrap_or_else(|| "UNKNOWN".to_string()),
                side: order.side.unwrap_or_else(|| "SELL".to_string()),
                quantity,
                price,
                total_usdt: price * quantity,
                timestamp,
                status: order.status.unwrap_or_else(|| "UNKNOWN".to_string()),
            }
        }));

        entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        entries.truncate(200);

        Ok(entries)
    }

    /// Get trading summary
    pub async fn get_trading_summary(
        &self,
        _ctx: &SecurityContext,
    ) -> Result<TradingSummary, TradingCoreError> {
        let conn = self.binance_conn().await?;
        let backend = conn.get_database_backend();

        let summary_stmt = Statement::from_sql_and_values(
            backend,
            r#"
            SELECT
                COUNT(*) AS total_trades,
                COUNT(*) FILTER (WHERE pnl > 0) AS winning_trades,
                COUNT(*) FILTER (WHERE pnl <= 0) AS losing_trades,
                COALESCE(SUM(pnl), 0) AS total_pnl,
                COALESCE(MAX(pnl), 0) AS largest_win,
                COALESCE(MIN(pnl), 0) AS largest_loss,
                COALESCE(AVG(CASE WHEN pnl > 0 THEN pnl END), 0) AS average_win,
                COALESCE(AVG(CASE WHEN pnl <= 0 THEN pnl END), 0) AS average_loss
            FROM pnl
            "#,
            vec![],
        );

        let summary_row = conn
            .query_one(summary_stmt)
            .await
            .map_err(|e| TradingCoreError::Database(e.to_string()))?
            .ok_or_else(|| TradingCoreError::Database("Failed to load trading summary".to_string()))?;

        let total_trades: i64 = Self::row_get(&summary_row, "total_trades")
            .unwrap_or(0);
        let winning_trades: i64 = Self::row_get(&summary_row, "winning_trades")
            .unwrap_or(0);
        let losing_trades: i64 = Self::row_get(&summary_row, "losing_trades")
            .unwrap_or(0);
        let total_pnl: f64 = Self::row_get(&summary_row, "total_pnl")
            .unwrap_or(0.0);
        let largest_win: f64 = Self::row_get(&summary_row, "largest_win")
            .unwrap_or(0.0);
        let largest_loss: f64 = Self::row_get(&summary_row, "largest_loss")
            .unwrap_or(0.0);
        let average_win: f64 = Self::row_get(&summary_row, "average_win")
            .unwrap_or(0.0);
        let average_loss: f64 = Self::row_get(&summary_row, "average_loss")
            .unwrap_or(0.0);

        let today = chrono::Utc::now().date_naive();
        let daily_stmt = Statement::from_sql_and_values(
            backend,
            r#"
            SELECT COALESCE(SUM(pnl), 0) AS daily_pnl
            FROM pnl
            WHERE timestamp::date = $1
            "#,
            vec![today.into()],
        );

        let daily_row = conn
            .query_one(daily_stmt)
            .await
            .map_err(|e| TradingCoreError::Database(e.to_string()))?
            .ok_or_else(|| TradingCoreError::Database("Failed to load daily pnl".to_string()))?;

        let daily_pnl: f64 = Self::row_get(&daily_row, "daily_pnl")
            .unwrap_or(0.0);

        let win_rate = if total_trades > 0 {
            (winning_trades as f64 / total_trades as f64) * 100.0
        } else {
            0.0
        };

        Ok(TradingSummary {
            total_trades,
            winning_trades,
            losing_trades,
            win_rate,
            total_pnl,
            daily_pnl,
            largest_win,
            largest_loss,
            average_win,
            average_loss,
        })
    }

    /// Health check
    pub async fn is_healthy(&self) -> bool {
        let db = self.db_binance.lock().await;
        db.is_some()
    }
}
