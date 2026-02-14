use std::collections::HashMap;
use std::sync::Arc;

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use modkit_db::DBProvider;
use modkit_db::DbError;
use modkit_db::secure::{AccessScope, SecureEntityExt};
use modkit_security::SecurityContext;
use tenant_resolver_sdk::TenantResolverGatewayClient;
use tracing::info;

use crate::config::TradingDashboardConfig;
use crate::infra::storage::entity::{buy_orders, pnl, sell_orders, temps, total_asset_val, trade_pairs};
use crate::api::rest::dto::{
    AccountBalancesDto, AssetValuePointDto, DailyPnlDto, DailyPnlListDto,
    OpenOrderDto, OpenOrdersListDto, PairsEvolutionDto, RecentTradeDto,
    RecentTradesListDto, TopPairDto, TopPairsListDto, TotalAssetsValueDto,
    TradingStatisticsDto,
};
use trading_dashboard_sdk::errors::TradingDashboardError;
use trading_dashboard_sdk::models::StatsSummary;

fn db_err(e: impl std::fmt::Display) -> TradingDashboardError {
    TradingDashboardError::Database(e.to_string())
}

/// Core service containing all trading dashboard business logic.
#[derive(Clone)]
pub struct TradingDashboardService {
    db: Arc<DBProvider<DbError>>,
    #[allow(dead_code)]
    resolver: Arc<dyn TenantResolverGatewayClient>,
    #[allow(dead_code)]
    config: TradingDashboardConfig,
}

impl TradingDashboardService {
    #[must_use]
    pub fn new(
        db: Arc<DBProvider<DbError>>,
        resolver: Arc<dyn TenantResolverGatewayClient>,
        config: TradingDashboardConfig,
    ) -> Self {
        Self {
            db,
            resolver,
            config,
        }
    }

    /// Get aggregated trading statistics summary.
    ///
    /// Queries across pnl, buy_orders, sell_orders, temps, and total_asset_val tables.
    /// All entities are `unrestricted` (no tenant scoping yet).
    pub async fn get_stats_summary(
        &self,
        ctx: &SecurityContext,
        _time_range: &str,
    ) -> Result<StatsSummary, TradingDashboardError> {
        info!(
            subject_id = %ctx.subject_id(),
            "Fetching trading stats summary"
        );

        let conn = self.db.conn().map_err(db_err)?;
        // Unrestricted entities use root scope — no tenant filtering applied
        // For unrestricted entities, scope value is ignored — use default
        let scope = AccessScope::default();

        // Count open buy orders
        let open_orders = buy_orders::Entity::find()
            .filter(buy_orders::Column::Status.eq("NEW"))
            .secure()
            .scope_with(&scope)
            .count(&conn)
            .await
            .map_err(db_err)?;

        // Count all completed trades (sell orders)
        let total_trades = sell_orders::Entity::find()
            .secure()
            .scope_with(&scope)
            .count(&conn)
            .await
            .map_err(db_err)?;

        // Get latest temps row for available_usdc and trading_enabled
        let temps_row = temps::Entity::find()
            .secure()
            .scope_with(&scope)
            .one(&conn)
            .await
            .map_err(db_err)?;

        // Get latest total asset value
        let asset_val = total_asset_val::Entity::find()
            .order_by_desc(total_asset_val::Column::RecordDate)
            .secure()
            .scope_with(&scope)
            .one(&conn)
            .await
            .map_err(db_err)?;

        // Sum PnL — use secure .all() and sum in Rust (simpler than custom column projection through secure layer)
        let all_pnl = pnl::Entity::find()
            .secure()
            .scope_with(&scope)
            .all(&conn)
            .await
            .map_err(db_err)?;

        let total_pnl: f64 = all_pnl.iter().filter_map(|p| p.pnl).sum();

        let (available_usdc, trading_enabled) = match temps_row {
            Some(t) => (
                t.available_usdc.unwrap_or(0.0),
                t.trading_enabled.unwrap_or(0) == 1,
            ),
            None => (0.0, false),
        };

        #[allow(clippy::cast_possible_wrap)]
        Ok(StatsSummary {
            total_trades: total_trades as i64,
            open_orders: open_orders as i64,
            total_pnl,
            total_asset_value: asset_val.map_or(0.0, |v| v.total_val.unwrap_or(0.0)),
            available_usdc,
            trading_enabled,
        })
    }

    /// Get daily P&L aggregation from the pnl table.
    pub async fn get_pnl_by_day(
        &self,
        ctx: &SecurityContext,
    ) -> Result<DailyPnlListDto, TradingDashboardError> {
        info!(subject_id = %ctx.subject_id(), "Fetching daily P&L");
        let conn = self.db.conn().map_err(db_err)?;
        let scope = AccessScope::default();

        let all_pnl = pnl::Entity::find()
            .filter(pnl::Column::Pnl.is_not_null())
            .order_by_desc(pnl::Column::Timestamp)
            .secure()
            .scope_with(&scope)
            .all(&conn)
            .await
            .map_err(db_err)?;

        let mut day_map: HashMap<String, (i64, i64, i64, f64)> = HashMap::new();
        for row in &all_pnl {
            let date_str = row.timestamp
                .map(|t| t.format("%Y-%m-%d").to_string())
                .unwrap_or_default();
            if date_str.is_empty() { continue; }
            let entry = day_map.entry(date_str).or_insert((0, 0, 0, 0.0));
            let p = row.pnl.unwrap_or(0.0);
            entry.0 += 1;
            if p >= 0.0 { entry.1 += 1; } else { entry.2 += 1; }
            entry.3 += p;
        }

        let mut daily: Vec<DailyPnlDto> = day_map
            .into_iter()
            .map(|(date, (trades, wins, losses, daily_pnl))| DailyPnlDto {
                date,
                trades,
                wins,
                losses,
                daily_pnl,
            })
            .collect();
        daily.sort_by(|a, b| b.date.cmp(&a.date));

        Ok(DailyPnlListDto { daily_pnl: daily })
    }

    /// Get recent trades (from pnl table joined with sell_orders for symbol).
    pub async fn get_recent_trades(
        &self,
        ctx: &SecurityContext,
        limit: u64,
    ) -> Result<RecentTradesListDto, TradingDashboardError> {
        info!(subject_id = %ctx.subject_id(), limit, "Fetching recent trades");
        let conn = self.db.conn().map_err(db_err)?;
        let scope = AccessScope::default();

        // Get recent pnl entries (these have timestamps and client_order_ids)
        let pnl_rows = pnl::Entity::find()
            .filter(pnl::Column::Pnl.is_not_null())
            .order_by_desc(pnl::Column::Timestamp)
            .secure()
            .scope_with(&scope)
            .all(&conn)
            .await
            .map_err(db_err)?;

        // Build a lookup of client_order_id -> symbol from sell_orders
        let sell_rows = sell_orders::Entity::find()
            .secure()
            .scope_with(&scope)
            .all(&conn)
            .await
            .map_err(db_err)?;

        let symbol_map: HashMap<String, String> = sell_rows
            .into_iter()
            .filter_map(|s| {
                let coid = s.client_order_id?;
                let sym = s.symbol?;
                Some((coid, sym))
            })
            .collect();

        let trades: Vec<RecentTradeDto> = pnl_rows
            .into_iter()
            .take(limit as usize)
            .map(|row| {
                let coid = row.client_order_id.clone().unwrap_or_default();
                let symbol = symbol_map.get(&coid).cloned();
                RecentTradeDto {
                    timestamp: row.timestamp.map(|t| t.format("%Y-%m-%dT%H:%M:%S%.6f").to_string()),
                    symbol,
                    pnl: row.pnl,
                    client_order_id: row.client_order_id,
                }
            })
            .collect();

        Ok(RecentTradesListDto { trades })
    }

    /// Get aggregate trading statistics.
    pub async fn get_statistics(
        &self,
        ctx: &SecurityContext,
    ) -> Result<TradingStatisticsDto, TradingDashboardError> {
        info!(subject_id = %ctx.subject_id(), "Fetching trading statistics");
        let conn = self.db.conn().map_err(db_err)?;
        let scope = AccessScope::default();

        let all_pnl_rows = pnl::Entity::find()
            .filter(pnl::Column::Pnl.is_not_null())
            .secure()
            .scope_with(&scope)
            .all(&conn)
            .await
            .map_err(db_err)?;

        let temps_row = temps::Entity::find()
            .secure()
            .scope_with(&scope)
            .one(&conn)
            .await
            .map_err(db_err)?;

        let trade_pair_rows = trade_pairs::Entity::find()
            .secure()
            .scope_with(&scope)
            .all(&conn)
            .await
            .map_err(db_err)?;

        let number_of_trades = all_pnl_rows.len() as i64;
        let total_pnl: f64 = all_pnl_rows.iter().filter_map(|r| r.pnl).sum();

        let mut unique_days = std::collections::HashSet::new();
        for r in &all_pnl_rows {
            if let Some(t) = r.timestamp {
                unique_days.insert(t.format("%Y-%m-%d").to_string());
            }
        }
        let unique_days_count = unique_days.len() as i64;

        let avg_pnl_per_trade = if number_of_trades > 0 {
            (total_pnl * 100.0).round() / 100.0 / number_of_trades as f64
        } else { 0.0 };
        let avg_pnl_per_day = if unique_days_count > 0 {
            (total_pnl * 100.0).round() / 100.0 / unique_days_count as f64
        } else { 0.0 };
        let trades_per_day = if unique_days_count > 0 {
            (number_of_trades as f64 / unique_days_count as f64 * 10.0).round() / 10.0
        } else { 0.0 };

        let (available_usdc, available_bnb) = match &temps_row {
            Some(t) => (t.available_usdc.unwrap_or(0.0), t.bnb.unwrap_or(0.0)),
            None => (0.0, 0.0),
        };

        // Pair evolution: count positive vs negative price changes
        let (pos_day, neg_day) = trade_pair_rows.iter().fold((0i64, 0i64), |(p, n), tp| {
            let start = tp.start_price_day.unwrap_or(0.0);
            let end = tp.end_price_day.unwrap_or(0.0);
            if end >= start { (p + 1, n) } else { (p, n + 1) }
        });
        let (pos_hour, neg_hour) = trade_pair_rows.iter().fold((0i64, 0i64), |(p, n), tp| {
            let start = tp.start_price_hour.unwrap_or(0.0);
            let end = tp.end_price_hour.unwrap_or(0.0);
            if end >= start { (p + 1, n) } else { (p, n + 1) }
        });

        Ok(TradingStatisticsDto {
            total_pnl: (total_pnl * 100.0).round() / 100.0,
            number_of_trades,
            unique_days: unique_days_count,
            avg_pnl_per_trade: (avg_pnl_per_trade * 100.0).round() / 100.0,
            avg_pnl_per_day: (avg_pnl_per_day * 100.0).round() / 100.0,
            trades_per_day,
            available_usdc,
            available_bnb,
            pairs_evolution_day: PairsEvolutionDto { positive: pos_day, negative: neg_day },
            pairs_evolution_hour: PairsEvolutionDto { positive: pos_hour, negative: neg_hour },
        })
    }

    /// Get top trading pairs by total P&L.
    pub async fn get_top_pairs(
        &self,
        ctx: &SecurityContext,
    ) -> Result<TopPairsListDto, TradingDashboardError> {
        info!(subject_id = %ctx.subject_id(), "Fetching top pairs");
        let conn = self.db.conn().map_err(db_err)?;
        let scope = AccessScope::default();

        let all_pnl_rows = pnl::Entity::find()
            .filter(pnl::Column::Pnl.is_not_null())
            .secure()
            .scope_with(&scope)
            .all(&conn)
            .await
            .map_err(db_err)?;

        // Build client_order_id -> symbol lookup
        let sell_rows = sell_orders::Entity::find()
            .secure()
            .scope_with(&scope)
            .all(&conn)
            .await
            .map_err(db_err)?;

        let symbol_map: HashMap<String, String> = sell_rows
            .into_iter()
            .filter_map(|s| {
                let coid = s.client_order_id?;
                let sym = s.symbol?;
                Some((coid, sym))
            })
            .collect();

        let mut pair_map: HashMap<String, (i64, i64, i64, f64)> = HashMap::new();
        for row in &all_pnl_rows {
            let coid = row.client_order_id.clone().unwrap_or_default();
            let sym = symbol_map.get(&coid).cloned().unwrap_or_default();
            if sym.is_empty() { continue; }
            let entry = pair_map.entry(sym).or_insert((0, 0, 0, 0.0));
            let p = row.pnl.unwrap_or(0.0);
            entry.0 += 1;
            if p >= 0.0 { entry.1 += 1; } else { entry.2 += 1; }
            entry.3 += p;
        }

        let mut pairs: Vec<TopPairDto> = pair_map
            .into_iter()
            .map(|(symbol, (total_trades, wins, losses, total_pnl))| {
                let win_rate = if total_trades > 0 {
                    (wins as f64 / total_trades as f64 * 1000.0).round() / 10.0
                } else { 0.0 };
                TopPairDto { symbol, total_trades, wins, losses, total_pnl, win_rate }
            })
            .collect();
        pairs.sort_by(|a, b| b.total_pnl.partial_cmp(&a.total_pnl).unwrap_or(std::cmp::Ordering::Equal));

        Ok(TopPairsListDto { pairs })
    }

    /// Get total asset value history (last 24h hourly).
    pub async fn get_total_assets_value(
        &self,
        ctx: &SecurityContext,
    ) -> Result<TotalAssetsValueDto, TradingDashboardError> {
        info!(subject_id = %ctx.subject_id(), "Fetching total assets value");
        let conn = self.db.conn().map_err(db_err)?;
        let scope = AccessScope::default();

        let rows = total_asset_val::Entity::find()
            .order_by_desc(total_asset_val::Column::RecordDate)
            .secure()
            .scope_with(&scope)
            .all(&conn)
            .await
            .map_err(db_err)?;

        // Take last 24 hourly entries
        let hourly: Vec<AssetValuePointDto> = rows
            .into_iter()
            .take(24)
            .rev()
            .map(|r| AssetValuePointDto {
                date: r.record_date
                    .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_default(),
                value: r.total_val.unwrap_or(0.0),
            })
            .collect();

        Ok(TotalAssetsValueDto { hourly_24h: hourly })
    }

    /// Get open buy orders.
    pub async fn get_open_orders(
        &self,
        ctx: &SecurityContext,
    ) -> Result<OpenOrdersListDto, TradingDashboardError> {
        info!(subject_id = %ctx.subject_id(), "Fetching open orders");
        let conn = self.db.conn().map_err(db_err)?;
        let scope = AccessScope::default();

        let rows = buy_orders::Entity::find()
            .filter(buy_orders::Column::Status.eq("NEW"))
            .order_by_desc(buy_orders::Column::TransactTime)
            .secure()
            .scope_with(&scope)
            .all(&conn)
            .await
            .map_err(db_err)?;

        let orders: Vec<OpenOrderDto> = rows
            .into_iter()
            .map(|b| OpenOrderDto {
                id: b.id,
                symbol: b.symbol,
                client_order_id: b.client_order_id,
                transact_time: b.transact_time.map(|t| t.format("%Y-%m-%dT%H:%M:%S%.6f").to_string()),
                price: b.price,
                qty: b.qty,
                status: b.status,
                est_profit_percent: b.est_profit_percent,
                highest_percentage_since_buy: b.highest_percentage_since_buy,
                lowest_percentage_since_buy: b.lowest_percentage_since_buy,
            })
            .collect();

        Ok(OpenOrdersListDto { orders })
    }

    /// Get account balances overview.
    pub async fn get_account_balances(
        &self,
        ctx: &SecurityContext,
    ) -> Result<AccountBalancesDto, TradingDashboardError> {
        info!(subject_id = %ctx.subject_id(), "Fetching account balances");
        let conn = self.db.conn().map_err(db_err)?;
        let scope = AccessScope::default();

        let temps_row = temps::Entity::find()
            .secure()
            .scope_with(&scope)
            .one(&conn)
            .await
            .map_err(db_err)?;

        let asset_val = total_asset_val::Entity::find()
            .order_by_desc(total_asset_val::Column::RecordDate)
            .secure()
            .scope_with(&scope)
            .one(&conn)
            .await
            .map_err(db_err)?;

        // Sum all buy order investments
        let all_buys = buy_orders::Entity::find()
            .filter(buy_orders::Column::Status.eq("NEW"))
            .secure()
            .scope_with(&scope)
            .all(&conn)
            .await
            .map_err(db_err)?;

        let invested: f64 = all_buys.iter()
            .filter_map(|b| b.cummulative_quote_qty)
            .sum();

        let available_usdc = temps_row.as_ref().map_or(0.0, |t| t.available_usdc.unwrap_or(0.0));
        let total_asset = asset_val.map_or(0.0, |v| v.total_val.unwrap_or(0.0));
        let total_value = available_usdc + total_asset;

        // Total invested = sum of all sell_orders cummulative_quote_qty (historical)
        let all_sells = sell_orders::Entity::find()
            .secure()
            .scope_with(&scope)
            .all(&conn)
            .await
            .map_err(db_err)?;

        let total_invested: f64 = all_sells.iter()
            .filter_map(|s| s.cummulative_quote_qty)
            .sum::<f64>()
            + invested;

        let profit_loss = total_value - total_invested;
        let profit_loss_percent = if total_invested > 0.0 {
            (profit_loss / total_invested * 10000.0).round() / 100.0
        } else { 0.0 };

        Ok(AccountBalancesDto {
            available_usdc,
            total_asset_val: total_asset,
            total_value: (total_value * 10000000.0).round() / 10000000.0,
            invested: (total_invested * 1000.0).round() / 1000.0,
            profit_loss: (profit_loss * 100.0).round() / 100.0,
            profit_loss_percent,
        })
    }
}
