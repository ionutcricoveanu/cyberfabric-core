//! Trading strategy orchestration — BuyManager and SellManager

use tracing::{info, warn, debug};
use modkit_security::SecurityContext;
use modkit_db::secure::{AccessScope, SecureEntityExt};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use std::sync::Arc;
use modkit_db::Db;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect};

use crate::domain::binance::BinanceApiClient;
use crate::domain::error::DomainError;
use crate::domain::models::{BuySignal, Order, Quote, AccountBalance};
use crate::domain::entities::{buy_orders, pnl};
use crate::config::TradingCoreConfig;

// ---------------------------------------------------------------------------
// BuyManager
// ---------------------------------------------------------------------------

/// Buy strategy manager — identifies entry opportunities and sizes positions.
pub struct BuyManager {
    config: Arc<TradingCoreConfig>,
    db_binance: Arc<Db>,
    #[allow(dead_code)]
    last_buy_times: std::sync::Mutex<std::collections::HashMap<String, std::time::Instant>>,
}

impl BuyManager {
    pub fn new(config: Arc<TradingCoreConfig>, db_binance: Arc<Db>) -> Self {
        Self {
            config,
            db_binance,
            last_buy_times: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }

    /// Evaluate whether to open a new position for `symbol`.
    ///
    /// Returns `Some(Order)` when a buy order should be placed, `None` otherwise.
    pub async fn execute_buy_strategy(
        &self,
        _ctx: &SecurityContext,
        symbol: &str,
        buy_signal: BuySignal,
        account_balance: &AccountBalance,
        current_quote: &Quote,
    ) -> Result<Option<Order>, DomainError> {
        info!(
            "Buy strategy: Processing {} — action={}, confidence={}",
            symbol, buy_signal.action, buy_signal.confidence
        );

        if !self.config.trading_enabled || !self.config.buying_enabled {
            warn!(
                "Buy strategy skipped — trading_enabled={}, buying_enabled={}",
                self.config.trading_enabled, self.config.buying_enabled
            );
            return Ok(None);
        }

        if buy_signal.action.to_uppercase() != "BUY" {
            debug!("Buy strategy: Skipping non-BUY action: {}", buy_signal.action);
            return Ok(None);
        }

        if buy_signal.confidence < 0.5 {
            debug!("Buy strategy: Confidence too low: {} < 0.5", buy_signal.confidence);
            return Ok(None);
        }

        // Don't open a second position for the same symbol
        if self.has_open_position(symbol).await? {
            debug!("Buy strategy: Position already exists for {}", symbol);
            return Ok(None);
        }

        // Enforce max concurrent positions
        let open_count = self.count_open_positions().await?;
        if open_count >= self.config.risk_management.max_open_positions as u64 {
            warn!(
                "Buy strategy: Max open positions reached ({}/{})",
                open_count, self.config.risk_management.max_open_positions
            );
            return Ok(None);
        }

        let position_size = self.calculate_position_size(
            account_balance,
            current_quote.bid,
            buy_signal.confidence,
        )?;

        let min_size = Decimal::from_f64_retain(self.config.position_sizing.min_position_usdt)
            .unwrap_or(Decimal::from(10));

        if position_size < min_size {
            warn!("Buy strategy: Position size {} below minimum {}", position_size, min_size);
            return Ok(None);
        }

        let quantity = position_size / current_quote.bid;

        let order = Order {
            symbol: symbol.to_string(),
            side: "BUY".to_string(),
            quantity: quantity.to_f64().unwrap_or(0.0),
            price: current_quote.bid,
            order_type: "LIMIT".to_string(),
            time_in_force: "GTC".to_string(),
        };

        info!(
            "Buy strategy: Generated order for {} — qty={:.8}, price={}",
            symbol, order.quantity, order.price
        );

        Ok(Some(order))
    }

    /// Kelly-criterion-based position sizing with max-position cap.
    fn calculate_position_size(
        &self,
        account_balance: &AccountBalance,
        _current_price: Decimal,
        confidence: f64,
    ) -> Result<Decimal, DomainError> {
        if let Some(fixed_size) = self.config.position_sizing.fixed_position_usdt {
            return Ok(Decimal::from_f64_retain(fixed_size).unwrap_or(Decimal::from(100)));
        }

        let kelly_fraction = self.config.position_sizing.kelly_fraction;
        let max_position_pct = self.config.position_sizing.max_position_pct / 100.0;

        let portfolio = account_balance.total;
        let max_position = portfolio
            * Decimal::from_f64_retain(max_position_pct).unwrap_or(Decimal::from_f64_retain(0.05).unwrap_or(Decimal::ONE));

        let kelly_position = portfolio
            * Decimal::from_f64_retain(kelly_fraction).unwrap_or(Decimal::from_f64_retain(0.25).unwrap_or(Decimal::ZERO))
            * Decimal::from_f64_retain(confidence).unwrap_or(Decimal::ONE);

        let position_size = kelly_position.min(max_position);

        debug!(
            "Buy strategy: portfolio={}, kelly={}, max={}, final={}",
            portfolio, kelly_position, max_position, position_size
        );

        Ok(position_size)
    }

    /// Check whether an open (status=NEW) buy order exists for `symbol`.
    async fn has_open_position(&self, symbol: &str) -> Result<bool, DomainError> {
        let conn = self
            .db_binance
            .conn()
            .map_err(|e| DomainError::Database(format!("DB connection error: {e}")))?;

        let scope = AccessScope::default();
        let count = buy_orders::Entity::find()
            .filter(buy_orders::Column::Status.eq("NEW"))
            .filter(buy_orders::Column::Symbol.eq(symbol))
            .secure()
            .scope_with(&scope)
            .count(&conn)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(count > 0)
    }

    /// Count total open (status=NEW) positions across all symbols.
    async fn count_open_positions(&self) -> Result<u64, DomainError> {
        let conn = self
            .db_binance
            .conn()
            .map_err(|e| DomainError::Database(format!("DB connection error: {e}")))?;

        let scope = AccessScope::default();
        let count = buy_orders::Entity::find()
            .filter(buy_orders::Column::Status.eq("NEW"))
            .secure()
            .scope_with(&scope)
            .count(&conn)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(count)
    }
}

// ---------------------------------------------------------------------------
// SellManager
// ---------------------------------------------------------------------------

/// Sell strategy manager — reviews open positions and decides when to exit.
pub struct SellManager {
    config: Arc<TradingCoreConfig>,
    db_binance: Arc<Db>,
}

impl SellManager {
    pub fn new(config: Arc<TradingCoreConfig>, db_binance: Arc<Db>) -> Self {
        Self { config, db_binance }
    }

    /// Review all open positions and return sell orders for those that hit
    /// take-profit or stop-loss thresholds based on the **current market price**.
    pub async fn execute_sell_strategy(
        &self,
        _ctx: &SecurityContext,
        binance_client: &BinanceApiClient,
    ) -> Result<Vec<Order>, DomainError> {
        info!("Sell strategy: Starting position review");

        if self.is_daily_loss_limit_exceeded().await? {
            warn!("Sell strategy: Daily loss limit exceeded, pausing sells");
            return Ok(Vec::new());
        }

        let conn = self
            .db_binance
            .conn()
            .map_err(|e| DomainError::Database(format!("DB connection error: {e}")))?;
        let scope = AccessScope::default();

        let open_positions = buy_orders::Entity::find()
            .filter(buy_orders::Column::Status.eq("NEW"))
            .order_by_desc(buy_orders::Column::TransactTime)
            .secure()
            .scope_with(&scope)
            .all(&conn)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        let mut sell_orders_out = Vec::new();

        for position in open_positions {
            let symbol = position.symbol.clone().unwrap_or_else(|| "UNKNOWN".to_string());
            let entry_price_f = position.order_price.or(position.price).unwrap_or(0.0);
            let quantity = position.qty.or(position.executed_qty).or(position.orig_qty).unwrap_or(0.0);

            let entry_price = Decimal::from_f64_retain(entry_price_f).unwrap_or(Decimal::ZERO);

            // Fetch live market price; fall back to entry price if unavailable
            let current_price = match binance_client.get_quote(&symbol).await {
                Ok(quote) => quote.bid,
                Err(e) => {
                    warn!("Sell strategy: Could not fetch quote for {}: {}", symbol, e);
                    entry_price  // conservative: no action without real price
                }
            };

            if let Some(reason) = self
                .should_close_position(&symbol, entry_price, current_price, quantity)
                .await?
            {
                info!("Sell strategy: Closing {} — reason={}", symbol, reason);
                sell_orders_out.push(Order {
                    symbol,
                    side: "SELL".to_string(),
                    quantity,
                    price: current_price,
                    order_type: "LIMIT".to_string(),
                    time_in_force: "GTC".to_string(),
                });
            }
        }

        info!(
            "Sell strategy: {} sell orders generated",
            sell_orders_out.len()
        );
        Ok(sell_orders_out)
    }

    /// Decide whether a position should be closed.
    /// Returns the close reason string or `None` to hold.
    async fn should_close_position(
        &self,
        symbol: &str,
        entry_price: Decimal,
        current_price: Decimal,
        _quantity: f64,
    ) -> Result<Option<String>, DomainError> {
        if entry_price == Decimal::ZERO {
            return Ok(None);
        }

        let pnl_pct = ((current_price - entry_price) / entry_price * Decimal::from(100))
            .to_f64()
            .unwrap_or(0.0);

        if pnl_pct >= self.config.risk_management.take_profit_pct {
            info!(
                "Sell strategy: Take profit hit for {} — PnL%={:.2}",
                symbol, pnl_pct
            );
            return Ok(Some("TAKE_PROFIT".to_string()));
        }

        if pnl_pct <= -self.config.risk_management.stop_loss_pct {
            warn!(
                "Sell strategy: Stop loss hit for {} — PnL%={:.2}",
                symbol, pnl_pct
            );
            return Ok(Some("STOP_LOSS".to_string()));
        }

        Ok(None)
    }

    /// Sum today's P&L from the `pnl` table.
    async fn get_daily_pnl(&self) -> Result<Decimal, DomainError> {
        let conn = self
            .db_binance
            .conn()
            .map_err(|e| DomainError::Database(format!("DB connection error: {e}")))?;
        let scope = AccessScope::default();

        let today = chrono::Utc::now().date_naive();
        let rows = pnl::Entity::find()
            .order_by_desc(pnl::Column::Timestamp)
            .limit(1000)
            .secure()
            .scope_with(&scope)
            .all(&conn)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        let daily_pnl: f64 = rows
            .iter()
            .filter(|r| r.timestamp.map(|t| t.date() == today).unwrap_or(false))
            .filter_map(|r| r.pnl)
            .sum();

        Ok(Decimal::from_f64_retain(daily_pnl).unwrap_or(Decimal::ZERO))
    }

    /// Returns `true` when the daily loss limit has been breached.
    async fn is_daily_loss_limit_exceeded(&self) -> Result<bool, DomainError> {
        let daily_pnl = self.get_daily_pnl().await?;
        let limit = Decimal::from_f64_retain(-self.config.risk_management.max_daily_loss_usdt)
            .unwrap_or(Decimal::from(-500));

        if daily_pnl < limit {
            warn!(
                "Sell strategy: Daily loss limit breached — PnL={} < limit={}",
                daily_pnl, limit
            );
            return Ok(true);
        }
        Ok(false)
    }
}
