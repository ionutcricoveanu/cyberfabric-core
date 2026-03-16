//! Database persistence helpers for recording placed orders and P&L.

use tracing::{info, error};
use sea_orm::{ActiveValue::Set, EntityTrait};
use modkit_db::Db;
use modkit_db::secure::{AccessScope, SecureInsertExt};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

use crate::domain::binance::PlacedOrder;
use crate::domain::entities::{buy_orders, sell_orders, pnl};
use crate::domain::error::DomainError;

/// Persist a placed buy order to the `buy_orders` table.
///
/// Returns the database-assigned primary key.
pub async fn record_buy_order(db: &Db, placed: &PlacedOrder) -> Result<i32, DomainError> {
    let conn = db
        .conn()
        .map_err(|e| DomainError::Database(format!("DB connection error: {e}")))?;
    let scope = AccessScope::default();

    let transact_time = chrono::DateTime::from_timestamp_millis(placed.transact_time_ms)
        .map(|dt| dt.naive_utc())
        .unwrap_or_else(|| chrono::Utc::now().naive_utc());

    let order_id_decimal = placed
        .order_id
        .parse::<i64>()
        .ok()
        .map(|id| Decimal::from(id));

    let price_f64 = placed.price.to_f64().unwrap_or(0.0);

    let active = buy_orders::ActiveModel {
        symbol: Set(Some(placed.symbol.clone())),
        order_id: Set(order_id_decimal),
        client_order_id: Set(Some(placed.client_order_id.clone())),
        transact_time: Set(Some(transact_time)),
        orig_qty: Set(Some(placed.orig_qty)),
        executed_qty: Set(Some(placed.executed_qty)),
        status: Set(Some(placed.status.clone())),
        side: Set(Some(placed.side.clone())),
        price: Set(Some(price_f64)),
        order_price: Set(Some(price_f64)),
        r#type: Set(Some("LIMIT".to_string())),
        time_in_force: Set(Some("GTC".to_string())),
        has_sell_order: Set(Some("FALSE".to_string())),
        can_sell: Set(Some(false)),
        ..Default::default()
    };

    let result = buy_orders::Entity::insert(active)
        .secure()
        .scope_with(&scope)
        .map_err(|e| DomainError::Database(format!("Scope error: {e}")))?
        .exec(&conn)
        .await
        .map_err(|e| DomainError::Database(format!("Failed to insert buy order: {e}")))?;

    let insert_id: i32 = result.last_insert_id;

    info!(
        "Persisted buy order: db_id={} exchange_id={} symbol={}",
        insert_id, placed.order_id, placed.symbol
    );
    Ok(insert_id)
}

/// Persist a placed sell order to the `sell_orders` table and record P&L.
///
/// * `buy_order_exchange_id` — exchange-side ID of the matching buy order (if known).
/// * `entry_price` — price at which the position was originally opened.
/// * `pnl_value` — realised P&L in USDT (positive = profit, negative = loss).
pub async fn record_sell_order(
    db: &Db,
    placed: &PlacedOrder,
    buy_order_exchange_id: Option<i64>,
    entry_price: Decimal,
    pnl_value: f64,
) -> Result<i32, DomainError> {
    let conn = db
        .conn()
        .map_err(|e| DomainError::Database(format!("DB connection error: {e}")))?;
    let scope = AccessScope::default();

    let transact_time = chrono::DateTime::from_timestamp_millis(placed.transact_time_ms)
        .map(|dt| dt.naive_utc())
        .unwrap_or_else(|| chrono::Utc::now().naive_utc());

    let order_id_decimal = placed
        .order_id
        .parse::<i64>()
        .ok()
        .map(|id| Decimal::from(id));

    let buy_order_decimal = buy_order_exchange_id.map(Decimal::from);
    let entry_price_f64 = entry_price.to_f64().unwrap_or(0.0);
    let sell_price_f64 = placed.price.to_f64().unwrap_or(0.0);

    let active = sell_orders::ActiveModel {
        symbol: Set(Some(placed.symbol.clone())),
        order_id: Set(order_id_decimal),
        client_order_id: Set(Some(placed.client_order_id.clone())),
        transact_time: Set(Some(transact_time)),
        orig_qty: Set(Some(placed.orig_qty)),
        executed_qty: Set(Some(placed.executed_qty)),
        status: Set(Some(placed.status.clone())),
        side: Set(Some("SELL".to_string())),
        price: Set(Some(sell_price_f64)),
        order_price: Set(Some(entry_price_f64)),
        pnl: Set(Some(pnl_value)),
        buy_order_id: Set(buy_order_decimal),
        r#type: Set(Some("LIMIT".to_string())),
        time_in_force: Set(Some("GTC".to_string())),
        ..Default::default()
    };

    let result = sell_orders::Entity::insert(active)
        .secure()
        .scope_with(&scope)
        .map_err(|e| DomainError::Database(format!("Scope error: {e}")))?
        .exec(&conn)
        .await
        .map_err(|e| DomainError::Database(format!("Failed to insert sell order: {e}")))?;

    let insert_id: i32 = result.last_insert_id;

    info!(
        "Persisted sell order: db_id={} exchange_id={} symbol={} pnl={:.4}",
        insert_id, placed.order_id, placed.symbol, pnl_value
    );

    // Also write a P&L record (non-fatal on failure)
    if let Err(e) = record_pnl(db, &placed.client_order_id, pnl_value).await {
        error!("Failed to record pnl for {}: {}", placed.symbol, e);
    }

    Ok(insert_id)
}

/// Write a row to the `pnl` table.
async fn record_pnl(db: &Db, client_order_id: &str, pnl_value: f64) -> Result<(), DomainError> {
    let conn = db
        .conn()
        .map_err(|e| DomainError::Database(format!("DB connection error: {e}")))?;
    let scope = AccessScope::default();

    let active = pnl::ActiveModel {
        timestamp: Set(Some(chrono::Utc::now().naive_utc())),
        client_order_id: Set(Some(client_order_id.to_string())),
        pnl: Set(Some(pnl_value)),
        ..Default::default()
    };

    let _ = pnl::Entity::insert(active)
        .secure()
        .scope_with(&scope)
        .map_err(|e| DomainError::Database(format!("Scope error: {e}")))?
        .exec(&conn)
        .await
        .map_err(|e| DomainError::Database(format!("Failed to insert pnl: {e}")))?;

    Ok(())
}
