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
use crate::infra::storage::entity::{buy_orders, df_view, pairs_exclusion_list, pnl, sell_orders, temps, total_asset_val, trade_pairs};
use crate::infra::storage::model_entity::{
    agent_config, agent_decisions, agent_performance, agent_trade_impact,
    llm_usage, market_sentiment, model_calibration_metrics,
    model_training_history, prediction_history_v2,
};
use crate::api::rest::dto::{
    AccountBalancesDto, AgentConfigDto, AgentConfigListDto, AgentDecisionDto,
    AgentDecisionTimelineDto, AgentDecisionTimelineListDto, AgentDecisionsListDto,
    AgentEffectivenessDto, AgentOverviewDto, AgentOverviewListDto,
    AgentPerformanceDto, AgentPerformanceListDto, AgentTradeImpactDto,
    AgentTradeImpactListDto, AssetValuePointDto, BucketDto,
    CalibrationMetricsSummaryDto, DailyPnlDto, DailyPnlListDto, DbTableDto,
    DbTablesListDto, ExcludedPairDto, ExcludedPairsListDto,
    LlmUsageByModelDto, LlmUsageDto, ModelAgeImpactDto, ModelAgeImpactListDto,
    ModelsSummaryDto, OpenOrderDto, OpenOrdersListDto, PairEvolutionItemDto,
    PairsEvolutionDto, PairsEvolutionListDto, PerformanceAlertDto,
    PerformanceAlertsListDto, PerformanceDistributionDto, PerformanceOverviewDto,
    PredictionDto, PredictionsListDto, RecentTradeDto, RecentTradesListDto,
    RegimeAnalysisDto, RegimeAnalysisListDto, RegimeCalibrationDto,
    RejectionReasonDto, RejectionReasonsListDto, SentimentTrendDto,
    SentimentTrendsListDto, StrategyComparisonDto,
    StrategyComparisonListDto, SymbolModelStatusDto, SymbolModelStatusListDto,
    TopPairDto, TopPairsListDto, TotalAssetsValueDto, TrainingHistoryDto,
    TrainingHistoryListDto, TrainingTimelineDto, TrainingTimelineListDto,
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
    /// Model_Data database (agents, models, predictions). None if not configured.
    model_db: Option<Arc<DBProvider<DbError>>>,
    #[allow(dead_code)]
    resolver: Arc<dyn TenantResolverGatewayClient>,
    #[allow(dead_code)]
    config: TradingDashboardConfig,
}

impl TradingDashboardService {
    #[must_use]
    pub fn new(
        db: Arc<DBProvider<DbError>>,
        model_db: Option<Arc<DBProvider<DbError>>>,
        resolver: Arc<dyn TenantResolverGatewayClient>,
        config: TradingDashboardConfig,
    ) -> Self {
        Self {
            db,
            model_db,
            resolver,
            config,
        }
    }

    /// Get the Model_Data DB connection, or error if not configured.
    fn model_conn(&self) -> Result<modkit_db::DbConn<'_>, TradingDashboardError> {
        self.model_db
            .as_ref()
            .ok_or_else(|| TradingDashboardError::Database("Model_Data database not configured".into()))?
            .conn()
            .map_err(db_err)
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

    // ================================================================
    // Trading Orders — pairs-evolution, excluded
    // ================================================================

    pub async fn get_pairs_evolution(
        &self,
        ctx: &SecurityContext,
    ) -> Result<PairsEvolutionListDto, TradingDashboardError> {
        info!(subject_id = %ctx.subject_id(), "Fetching pairs evolution");
        let conn = self.db.conn().map_err(db_err)?;
        let scope = AccessScope::default();

        let rows = df_view::Entity::find()
            .secure()
            .scope_with(&scope)
            .all(&conn)
            .await
            .map_err(db_err)?;

        let pairs = rows.into_iter().map(|r| PairEvolutionItemDto {
            pair: r.pair,
            highest_price: r.highest_price,
            current_price: r.current_price,
            lowest_price: r.lowes_price,
            highest_percentage: r.highest_percentage,
            lowest_percentage: r.lowest_percentage,
            one_day_change_percentage: r.one_day_change_percentage,
        }).collect();

        Ok(PairsEvolutionListDto { pairs })
    }

    pub async fn get_excluded_pairs(
        &self,
        ctx: &SecurityContext,
    ) -> Result<ExcludedPairsListDto, TradingDashboardError> {
        info!(subject_id = %ctx.subject_id(), "Fetching excluded pairs");
        let conn = self.db.conn().map_err(db_err)?;
        let scope = AccessScope::default();

        let rows = pairs_exclusion_list::Entity::find()
            .order_by_desc(pairs_exclusion_list::Column::AddedDate)
            .secure()
            .scope_with(&scope)
            .all(&conn)
            .await
            .map_err(db_err)?;

        let pairs = rows.into_iter().map(|r| ExcludedPairDto {
            id: r.id,
            pair: r.pair,
            added_date: r.added_date.map(|t| t.format("%Y-%m-%dT%H:%M:%S").to_string()),
            auto_remove: r.auto_remove,
        }).collect();

        Ok(ExcludedPairsListDto { pairs })
    }

    // ================================================================
    // Models — summary, training-history, rejection-reasons, etc.
    // ================================================================

    pub async fn get_models_summary(
        &self,
        ctx: &SecurityContext,
    ) -> Result<ModelsSummaryDto, TradingDashboardError> {
        info!(subject_id = %ctx.subject_id(), "Fetching models summary");
        let conn = self.model_conn()?;
        let scope = AccessScope::default();

        let all = model_training_history::Entity::find()
            .secure()
            .scope_with(&scope)
            .all(&conn)
            .await
            .map_err(db_err)?;

        let total = all.len() as i64;
        let promoted = all.iter().filter(|m| m.promoted).count() as i64;
        let rejected = total - promoted;
        let promotion_rate = if total > 0 { (promoted as f64 / total as f64 * 1000.0).round() / 10.0 } else { 0.0 };

        let unique_symbols: i64 = {
            let mut syms = std::collections::HashSet::new();
            for m in &all { syms.insert(m.symbol.clone()); }
            syms.len() as i64
        };

        let avg_win_rate = {
            let vals: Vec<f64> = all.iter().filter_map(|m| m.win_rate.map(|v| v.to_string().parse::<f64>().unwrap_or(0.0))).collect();
            if vals.is_empty() { 0.0 } else { (vals.iter().sum::<f64>() / vals.len() as f64 * 100.0).round() / 100.0 }
        };
        let avg_sharpe = {
            let vals: Vec<f64> = all.iter().filter_map(|m| m.sharpe_ratio.map(|v| v.to_string().parse::<f64>().unwrap_or(0.0))).collect();
            if vals.is_empty() { 0.0 } else { (vals.iter().sum::<f64>() / vals.len() as f64 * 100.0).round() / 100.0 }
        };

        Ok(ModelsSummaryDto {
            total_models_trained: total,
            promoted_models: promoted,
            rejected_models: rejected,
            promotion_rate,
            unique_symbols,
            avg_win_rate,
            avg_sharpe_ratio: avg_sharpe,
        })
    }

    pub async fn get_training_history(
        &self,
        ctx: &SecurityContext,
        limit: u64,
    ) -> Result<TrainingHistoryListDto, TradingDashboardError> {
        info!(subject_id = %ctx.subject_id(), limit, "Fetching training history");
        let conn = self.model_conn()?;
        let scope = AccessScope::default();

        let rows = model_training_history::Entity::find()
            .order_by_desc(model_training_history::Column::TrainedAt)
            .secure()
            .scope_with(&scope)
            .all(&conn)
            .await
            .map_err(db_err)?;

        let dec_to_f64 = |d: &sea_orm::prelude::Decimal| -> f64 { d.to_string().parse::<f64>().unwrap_or(0.0) };

        let records: Vec<TrainingHistoryDto> = rows.into_iter().take(limit as usize).map(|r| TrainingHistoryDto {
            id: r.id,
            symbol: r.symbol,
            trained_at: r.trained_at.format("%Y-%m-%dT%H:%M:%S").to_string(),
            promoted: r.promoted,
            win_rate: r.win_rate.as_ref().map(dec_to_f64),
            total_trades: r.total_trades,
            total_return: r.total_return.as_ref().map(dec_to_f64),
            sharpe_ratio: r.sharpe_ratio.as_ref().map(dec_to_f64),
            max_drawdown: r.max_drawdown.as_ref().map(dec_to_f64),
            profit_factor: r.profit_factor.as_ref().map(dec_to_f64),
            training_duration_seconds: r.training_duration_seconds.as_ref().map(dec_to_f64),
            rejection_reason: r.rejection_reason,
            interval: r.interval,
            wf_passed_gates: r.wf_passed_gates,
            oos_passed_gates: r.oos_passed_gates,
        }).collect();

        Ok(TrainingHistoryListDto { records })
    }

    pub async fn get_rejection_reasons(
        &self,
        ctx: &SecurityContext,
    ) -> Result<RejectionReasonsListDto, TradingDashboardError> {
        info!(subject_id = %ctx.subject_id(), "Fetching rejection reasons");
        let conn = self.model_conn()?;
        let scope = AccessScope::default();

        let rows = model_training_history::Entity::find()
            .filter(model_training_history::Column::Promoted.eq(false))
            .filter(model_training_history::Column::RejectionReason.is_not_null())
            .secure()
            .scope_with(&scope)
            .all(&conn)
            .await
            .map_err(db_err)?;

        let mut reason_map: HashMap<String, i64> = HashMap::new();
        for r in &rows {
            if let Some(reason) = &r.rejection_reason {
                *reason_map.entry(reason.clone()).or_insert(0) += 1;
            }
        }

        let mut reasons: Vec<RejectionReasonDto> = reason_map.into_iter()
            .map(|(reason, count)| RejectionReasonDto { reason, count })
            .collect();
        reasons.sort_by(|a, b| b.count.cmp(&a.count));

        Ok(RejectionReasonsListDto { reasons })
    }

    pub async fn get_symbol_model_status(
        &self,
        ctx: &SecurityContext,
    ) -> Result<SymbolModelStatusListDto, TradingDashboardError> {
        info!(subject_id = %ctx.subject_id(), "Fetching symbol model status");
        let conn = self.model_conn()?;
        let scope = AccessScope::default();

        let rows = model_training_history::Entity::find()
            .order_by_desc(model_training_history::Column::TrainedAt)
            .secure()
            .scope_with(&scope)
            .all(&conn)
            .await
            .map_err(db_err)?;

        let dec_to_f64 = |d: &sea_orm::prelude::Decimal| -> f64 { d.to_string().parse::<f64>().unwrap_or(0.0) };

        let mut symbol_map: HashMap<String, (i64, i64, Option<String>, Option<bool>, Option<f64>)> = HashMap::new();
        for r in &rows {
            let entry = symbol_map.entry(r.symbol.clone()).or_insert((0, 0, None, None, None));
            entry.0 += 1;
            if r.promoted { entry.1 += 1; }
            if entry.2.is_none() {
                entry.2 = Some(r.trained_at.format("%Y-%m-%dT%H:%M:%S").to_string());
                entry.3 = Some(r.promoted);
                entry.4 = r.win_rate.as_ref().map(dec_to_f64);
            }
        }

        let mut symbols: Vec<SymbolModelStatusDto> = symbol_map.into_iter()
            .map(|(symbol, (total_trained, promoted, latest_trained_at, latest_promoted, latest_win_rate))| {
                SymbolModelStatusDto { symbol, total_trained, promoted, latest_trained_at, latest_promoted, latest_win_rate }
            }).collect();
        symbols.sort_by(|a, b| b.total_trained.cmp(&a.total_trained));

        Ok(SymbolModelStatusListDto { symbols })
    }

    pub async fn get_training_timeline(
        &self,
        ctx: &SecurityContext,
    ) -> Result<TrainingTimelineListDto, TradingDashboardError> {
        info!(subject_id = %ctx.subject_id(), "Fetching training timeline");
        let conn = self.model_conn()?;
        let scope = AccessScope::default();

        let rows = model_training_history::Entity::find()
            .order_by_desc(model_training_history::Column::TrainedAt)
            .secure()
            .scope_with(&scope)
            .all(&conn)
            .await
            .map_err(db_err)?;

        let mut day_map: HashMap<String, (i64, i64, i64)> = HashMap::new();
        for r in &rows {
            let date = r.trained_at.format("%Y-%m-%d").to_string();
            let entry = day_map.entry(date).or_insert((0, 0, 0));
            entry.0 += 1;
            if r.promoted { entry.1 += 1; } else { entry.2 += 1; }
        }

        let mut timeline: Vec<TrainingTimelineDto> = day_map.into_iter()
            .map(|(date, (total_trained, promoted, rejected))| TrainingTimelineDto { date, total_trained, promoted, rejected })
            .collect();
        timeline.sort_by(|a, b| b.date.cmp(&a.date));

        Ok(TrainingTimelineListDto { timeline })
    }

    pub async fn get_performance_distribution(
        &self,
        ctx: &SecurityContext,
    ) -> Result<PerformanceDistributionDto, TradingDashboardError> {
        info!(subject_id = %ctx.subject_id(), "Fetching performance distribution");
        let conn = self.model_conn()?;
        let scope = AccessScope::default();

        let rows = model_training_history::Entity::find()
            .filter(model_training_history::Column::Promoted.eq(true))
            .secure()
            .scope_with(&scope)
            .all(&conn)
            .await
            .map_err(db_err)?;

        let dec_to_f64 = |d: &sea_orm::prelude::Decimal| -> f64 { d.to_string().parse::<f64>().unwrap_or(0.0) };

        let wr_buckets = vec![
            ("0-30%", 0.0, 0.30), ("30-40%", 0.30, 0.40), ("40-50%", 0.40, 0.50),
            ("50-60%", 0.50, 0.60), ("60-70%", 0.60, 0.70), ("70-100%", 0.70, 1.01),
        ];
        let sr_buckets = vec![
            ("<0", f64::MIN, 0.0), ("0-0.5", 0.0, 0.5), ("0.5-1.0", 0.5, 1.0),
            ("1.0-2.0", 1.0, 2.0), (">2.0", 2.0, f64::MAX),
        ];

        let win_rate_buckets: Vec<BucketDto> = wr_buckets.into_iter().map(|(range, lo, hi)| {
            let count = rows.iter().filter(|r| {
                let wr = r.win_rate.as_ref().map(dec_to_f64).unwrap_or(0.0);
                wr >= lo && wr < hi
            }).count() as i64;
            BucketDto { range: range.to_string(), count }
        }).collect();

        let sharpe_ratio_buckets: Vec<BucketDto> = sr_buckets.into_iter().map(|(range, lo, hi)| {
            let count = rows.iter().filter(|r| {
                let sr = r.sharpe_ratio.as_ref().map(dec_to_f64).unwrap_or(0.0);
                sr >= lo && sr < hi
            }).count() as i64;
            BucketDto { range: range.to_string(), count }
        }).collect();

        Ok(PerformanceDistributionDto { win_rate_buckets, sharpe_ratio_buckets })
    }

    pub async fn get_calibration_metrics(
        &self,
        ctx: &SecurityContext,
    ) -> Result<CalibrationMetricsSummaryDto, TradingDashboardError> {
        info!(subject_id = %ctx.subject_id(), "Fetching calibration metrics");
        let conn = self.model_conn()?;
        let scope = AccessScope::default();

        let rows = model_calibration_metrics::Entity::find()
            .secure()
            .scope_with(&scope)
            .all(&conn)
            .await
            .map_err(db_err)?;

        let dec_to_f64 = |d: &sea_orm::prelude::Decimal| -> f64 { d.to_string().parse::<f64>().unwrap_or(0.0) };

        let total = rows.len() as i64;
        let correct = rows.iter().filter(|r| r.direction_correct == Some(true)).count() as f64;
        let direction_accuracy = if total > 0 { (correct / total as f64 * 1000.0).round() / 10.0 } else { 0.0 };

        let avg_cal_error = {
            let vals: Vec<f64> = rows.iter().filter_map(|r| r.calibration_error.as_ref().map(dec_to_f64)).collect();
            if vals.is_empty() { 0.0 } else { (vals.iter().sum::<f64>() / vals.len() as f64 * 10000.0).round() / 10000.0 }
        };
        let avg_confidence = {
            let vals: Vec<f64> = rows.iter().map(|r| dec_to_f64(&r.confidence_score)).collect();
            if vals.is_empty() { 0.0 } else { (vals.iter().sum::<f64>() / vals.len() as f64 * 10000.0).round() / 10000.0 }
        };

        let mut regime_map: HashMap<String, (i64, i64, f64)> = HashMap::new();
        for r in &rows {
            let regime = r.market_regime.clone().unwrap_or_else(|| "unknown".to_string());
            let entry = regime_map.entry(regime).or_insert((0, 0, 0.0));
            entry.0 += 1;
            if r.direction_correct == Some(true) { entry.1 += 1; }
            entry.2 += dec_to_f64(&r.confidence_score);
        }

        let by_regime: Vec<RegimeCalibrationDto> = regime_map.into_iter().map(|(regime, (count, correct, conf_sum))| {
            RegimeCalibrationDto {
                regime,
                count,
                accuracy: if count > 0 { (correct as f64 / count as f64 * 1000.0).round() / 10.0 } else { 0.0 },
                avg_confidence: if count > 0 { (conf_sum / count as f64 * 10000.0).round() / 10000.0 } else { 0.0 },
            }
        }).collect();

        Ok(CalibrationMetricsSummaryDto { total_predictions: total, direction_accuracy, avg_calibration_error: avg_cal_error, avg_confidence, by_regime })
    }

    pub async fn get_regime_analysis(
        &self,
        ctx: &SecurityContext,
    ) -> Result<RegimeAnalysisListDto, TradingDashboardError> {
        info!(subject_id = %ctx.subject_id(), "Fetching regime analysis");
        let conn = self.model_conn()?;
        let scope = AccessScope::default();

        let rows = model_calibration_metrics::Entity::find()
            .secure()
            .scope_with(&scope)
            .all(&conn)
            .await
            .map_err(db_err)?;

        let dec_to_f64 = |d: &sea_orm::prelude::Decimal| -> f64 { d.to_string().parse::<f64>().unwrap_or(0.0) };

        let mut regime_map: HashMap<String, (i64, i64, f64, f64)> = HashMap::new();
        for r in &rows {
            let regime = r.market_regime.clone().unwrap_or_else(|| "unknown".to_string());
            let entry = regime_map.entry(regime).or_insert((0, 0, 0.0, 0.0));
            entry.0 += 1;
            if r.direction_correct == Some(true) { entry.1 += 1; }
            entry.2 += dec_to_f64(&r.confidence_score);
        }

        let regimes: Vec<RegimeAnalysisDto> = regime_map.into_iter().map(|(regime, (count, promoted, wr_sum, sharpe_sum))| {
            RegimeAnalysisDto {
                regime,
                count,
                promoted,
                avg_win_rate: if count > 0 { (wr_sum / count as f64 * 100.0).round() / 100.0 } else { 0.0 },
                avg_sharpe: if count > 0 { (sharpe_sum / count as f64 * 100.0).round() / 100.0 } else { 0.0 },
            }
        }).collect();

        Ok(RegimeAnalysisListDto { regimes })
    }

    pub async fn get_model_age_impact(
        &self,
        ctx: &SecurityContext,
    ) -> Result<ModelAgeImpactListDto, TradingDashboardError> {
        info!(subject_id = %ctx.subject_id(), "Fetching model age impact");
        let conn = self.model_conn()?;
        let scope = AccessScope::default();

        let rows = model_calibration_metrics::Entity::find()
            .filter(model_calibration_metrics::Column::ModelAgeDays.is_not_null())
            .secure()
            .scope_with(&scope)
            .all(&conn)
            .await
            .map_err(db_err)?;

        let dec_to_f64 = |d: &sea_orm::prelude::Decimal| -> f64 { d.to_string().parse::<f64>().unwrap_or(0.0) };

        let age_ranges = vec![
            ("0-7 days", 0, 7), ("8-14 days", 8, 14), ("15-30 days", 15, 30),
            ("31-60 days", 31, 60), ("60+ days", 61, 9999),
        ];

        let buckets: Vec<ModelAgeImpactDto> = age_ranges.into_iter().map(|(label, lo, hi)| {
            let matching: Vec<&model_calibration_metrics::Model> = rows.iter()
                .filter(|r| { let age = r.model_age_days.unwrap_or(0); age >= lo && age <= hi })
                .collect();
            let count = matching.len() as i64;
            let correct = matching.iter().filter(|r| r.direction_correct == Some(true)).count() as f64;
            let direction_accuracy = if count > 0 { (correct / count as f64 * 1000.0).round() / 10.0 } else { 0.0 };
            let avg_quality_score = {
                let vals: Vec<f64> = matching.iter().filter_map(|r| r.adjusted_confidence.as_ref().map(dec_to_f64)).collect();
                if vals.is_empty() { 0.0 } else { (vals.iter().sum::<f64>() / vals.len() as f64 * 10000.0).round() / 10000.0 }
            };
            ModelAgeImpactDto { age_bucket: label.to_string(), count, direction_accuracy, avg_quality_score }
        }).collect();

        Ok(ModelAgeImpactListDto { buckets })
    }

    // ================================================================
    // Predictions
    // ================================================================

    pub async fn get_predictions(
        &self,
        ctx: &SecurityContext,
        limit: u64,
    ) -> Result<PredictionsListDto, TradingDashboardError> {
        info!(subject_id = %ctx.subject_id(), limit, "Fetching predictions");
        let conn = self.model_conn()?;
        let scope = AccessScope::default();

        let rows = prediction_history_v2::Entity::find()
            .order_by_desc(prediction_history_v2::Column::PredictionTimestamp)
            .secure()
            .scope_with(&scope)
            .all(&conn)
            .await
            .map_err(db_err)?;

        let dec_to_f64 = |d: &sea_orm::prelude::Decimal| -> f64 { d.to_string().parse::<f64>().unwrap_or(0.0) };

        let predictions: Vec<PredictionDto> = rows.into_iter().take(limit as usize).map(|r| PredictionDto {
            id: r.id,
            symbol: r.symbol,
            prediction_timestamp: r.prediction_timestamp.map(|t| t.format("%Y-%m-%dT%H:%M:%S").to_string()),
            model_version: r.model_version,
            market_regime: r.market_regime,
            up_probability: r.up_probability.as_ref().map(dec_to_f64),
            down_probability: r.down_probability.as_ref().map(dec_to_f64),
            confidence_score: r.confidence_score.as_ref().map(dec_to_f64),
            predicted_profit_percent: r.predicted_profit_percent.as_ref().map(dec_to_f64),
            resulted_in_trade: r.resulted_in_trade,
            direction_correct: r.direction_correct,
            actual_profit_percent: r.actual_profit_percent.as_ref().map(dec_to_f64),
            prediction_quality_score: r.prediction_quality_score.as_ref().map(dec_to_f64),
            model_age_days: r.model_age_days,
        }).collect();

        Ok(PredictionsListDto { predictions })
    }

    // ================================================================
    // Agents — overview, decisions, timeline, sentiment, performance, llm, config
    // ================================================================

    pub async fn get_agents_overview(
        &self,
        ctx: &SecurityContext,
    ) -> Result<AgentOverviewListDto, TradingDashboardError> {
        info!(subject_id = %ctx.subject_id(), "Fetching agents overview");
        let conn = self.model_conn()?;
        let scope = AccessScope::default();

        let decisions = agent_decisions::Entity::find()
            .secure()
            .scope_with(&scope)
            .all(&conn)
            .await
            .map_err(db_err)?;

        let configs = agent_config::Entity::find()
            .secure()
            .scope_with(&scope)
            .all(&conn)
            .await
            .map_err(db_err)?;

        let config_map: HashMap<String, bool> = configs.into_iter()
            .map(|c| (c.agent_type, c.enabled.unwrap_or(false)))
            .collect();

        let now = chrono::Utc::now().naive_utc();
        let day_ago = now - chrono::Duration::hours(24);

        let mut agent_map: HashMap<String, (i64, i64, f64, f64)> = HashMap::new();
        for d in &decisions {
            let entry = agent_map.entry(d.agent_type.clone()).or_insert((0, 0, 0.0, 0.0));
            entry.0 += 1;
            if d.created_at.map_or(false, |t| t > day_ago) { entry.1 += 1; }
            entry.2 += d.confidence.unwrap_or(0.0);
            entry.3 += d.cost_usd.as_ref().map(|v| v.to_string().parse::<f64>().unwrap_or(0.0)).unwrap_or(0.0);
        }

        let agents: Vec<AgentOverviewDto> = agent_map.into_iter().map(|(agent_type, (total, recent, conf_sum, cost))| {
            AgentOverviewDto {
                enabled: *config_map.get(&agent_type).unwrap_or(&false),
                avg_confidence: if total > 0 { (conf_sum / total as f64 * 100.0).round() / 100.0 } else { 0.0 },
                total_cost_usd: (cost * 100.0).round() / 100.0,
                total_decisions: total,
                recent_decisions_24h: recent,
                agent_type,
            }
        }).collect();

        Ok(AgentOverviewListDto { agents })
    }

    pub async fn get_agent_decisions_recent(
        &self,
        ctx: &SecurityContext,
        limit: u64,
    ) -> Result<AgentDecisionsListDto, TradingDashboardError> {
        info!(subject_id = %ctx.subject_id(), limit, "Fetching recent agent decisions");
        let conn = self.model_conn()?;
        let scope = AccessScope::default();

        let rows = agent_decisions::Entity::find()
            .order_by_desc(agent_decisions::Column::CreatedAt)
            .secure()
            .scope_with(&scope)
            .all(&conn)
            .await
            .map_err(db_err)?;

        let decisions: Vec<AgentDecisionDto> = rows.into_iter().take(limit as usize).map(|r| AgentDecisionDto {
            id: r.id,
            agent_type: r.agent_type,
            decision_type: r.decision_type,
            symbol: r.symbol,
            recommendation: r.recommendation,
            confidence: r.confidence,
            status: r.status,
            created_at: r.created_at.map(|t| t.format("%Y-%m-%dT%H:%M:%S").to_string()),
            processing_time_ms: r.processing_time_ms,
            cost_usd: r.cost_usd.as_ref().map(|v| v.to_string().parse::<f64>().unwrap_or(0.0)),
            llm_model: r.llm_model,
        }).collect();

        Ok(AgentDecisionsListDto { decisions })
    }

    pub async fn get_agent_decisions_timeline(
        &self,
        ctx: &SecurityContext,
    ) -> Result<AgentDecisionTimelineListDto, TradingDashboardError> {
        info!(subject_id = %ctx.subject_id(), "Fetching agent decisions timeline");
        let conn = self.model_conn()?;
        let scope = AccessScope::default();

        let perf_rows = agent_performance::Entity::find()
            .order_by_desc(agent_performance::Column::Date)
            .secure()
            .scope_with(&scope)
            .all(&conn)
            .await
            .map_err(db_err)?;

        let mut day_map: HashMap<String, (i64, i64, i64)> = HashMap::new();
        for r in &perf_rows {
            let date = r.date.format("%Y-%m-%d").to_string();
            let entry = day_map.entry(date).or_insert((0, 0, 0));
            entry.0 += r.decisions_made.unwrap_or(0) as i64;
            entry.1 += r.decisions_applied.unwrap_or(0) as i64;
            entry.2 += r.decisions_rejected.unwrap_or(0) as i64;
        }

        let mut timeline: Vec<AgentDecisionTimelineDto> = day_map.into_iter()
            .map(|(date, (total, applied, rejected))| AgentDecisionTimelineDto { date, total, applied, rejected })
            .collect();
        timeline.sort_by(|a, b| b.date.cmp(&a.date));

        Ok(AgentDecisionTimelineListDto { timeline })
    }

    pub async fn get_sentiment_trends(
        &self,
        ctx: &SecurityContext,
    ) -> Result<SentimentTrendsListDto, TradingDashboardError> {
        info!(subject_id = %ctx.subject_id(), "Fetching sentiment trends");
        let conn = self.model_conn()?;
        let scope = AccessScope::default();

        let rows = market_sentiment::Entity::find()
            .order_by_desc(market_sentiment::Column::CreatedAt)
            .secure()
            .scope_with(&scope)
            .all(&conn)
            .await
            .map_err(db_err)?;

        let mut day_map: HashMap<String, (f64, i64)> = HashMap::new();
        for r in &rows {
            let date = r.created_at.map(|t| t.format("%Y-%m-%d").to_string()).unwrap_or_default();
            if date.is_empty() { continue; }
            let entry = day_map.entry(date).or_insert((0.0, 0));
            entry.0 += r.sentiment_score.unwrap_or(0.0);
            entry.1 += 1;
        }

        let mut trends: Vec<SentimentTrendDto> = day_map.into_iter().map(|(date, (score_sum, count))| {
            SentimentTrendDto { date, avg_score: (score_sum / count as f64 * 10000.0).round() / 10000.0, count }
        }).collect();
        trends.sort_by(|a, b| b.date.cmp(&a.date));

        Ok(SentimentTrendsListDto { trends })
    }

    pub async fn get_agent_performance_metrics(
        &self,
        ctx: &SecurityContext,
    ) -> Result<AgentPerformanceListDto, TradingDashboardError> {
        info!(subject_id = %ctx.subject_id(), "Fetching agent performance metrics");
        let conn = self.model_conn()?;
        let scope = AccessScope::default();

        let rows = agent_performance::Entity::find()
            .order_by_desc(agent_performance::Column::Date)
            .secure()
            .scope_with(&scope)
            .all(&conn)
            .await
            .map_err(db_err)?;

        let metrics: Vec<AgentPerformanceDto> = rows.into_iter().map(|r| AgentPerformanceDto {
            id: r.id,
            agent_type: r.agent_type,
            date: r.date.format("%Y-%m-%d").to_string(),
            decisions_made: r.decisions_made,
            decisions_applied: r.decisions_applied,
            decisions_rejected: r.decisions_rejected,
            avg_confidence: r.avg_confidence,
            avg_processing_time_ms: r.avg_processing_time_ms,
            total_cost_usd: r.total_cost_usd.as_ref().map(|v| v.to_string().parse::<f64>().unwrap_or(0.0)),
            positive_outcomes: r.positive_outcomes,
            negative_outcomes: r.negative_outcomes,
            neutral_outcomes: r.neutral_outcomes,
        }).collect();

        Ok(AgentPerformanceListDto { metrics })
    }

    pub async fn get_llm_usage(
        &self,
        ctx: &SecurityContext,
    ) -> Result<LlmUsageDto, TradingDashboardError> {
        info!(subject_id = %ctx.subject_id(), "Fetching LLM usage");
        let conn = self.model_conn()?;
        let scope = AccessScope::default();

        let rows = llm_usage::Entity::find()
            .secure()
            .scope_with(&scope)
            .all(&conn)
            .await
            .map_err(db_err)?;

        let total_requests = rows.len() as i64;
        let total_input_tokens: i64 = rows.iter().map(|r| r.input_tokens.unwrap_or(0) as i64).sum();
        let total_output_tokens: i64 = rows.iter().map(|r| r.output_tokens.unwrap_or(0) as i64).sum();
        let total_cost_usd: f64 = rows.iter().map(|r| r.cost_usd.as_ref().map(|v| v.to_string().parse::<f64>().unwrap_or(0.0)).unwrap_or(0.0)).sum();
        let total_time: f64 = rows.iter().map(|r| r.processing_time_ms.unwrap_or(0) as f64).sum();
        let avg_processing_time_ms = if total_requests > 0 { (total_time / total_requests as f64 * 100.0).round() / 100.0 } else { 0.0 };

        let mut model_map: HashMap<String, (i64, i64, f64)> = HashMap::new();
        for r in &rows {
            let entry = model_map.entry(r.llm_model.clone()).or_insert((0, 0, 0.0));
            entry.0 += 1;
            entry.1 += r.input_tokens.unwrap_or(0) as i64 + r.output_tokens.unwrap_or(0) as i64;
            entry.2 += r.cost_usd.as_ref().map(|v| v.to_string().parse::<f64>().unwrap_or(0.0)).unwrap_or(0.0);
        }

        let by_model: Vec<LlmUsageByModelDto> = model_map.into_iter().map(|(model, (requests, total_tokens, cost_usd))| {
            LlmUsageByModelDto { model, requests, total_tokens, cost_usd: (cost_usd * 1000000.0).round() / 1000000.0 }
        }).collect();

        Ok(LlmUsageDto { total_requests, total_input_tokens, total_output_tokens, total_cost_usd: (total_cost_usd * 1000000.0).round() / 1000000.0, avg_processing_time_ms, by_model })
    }

    pub async fn get_agent_config(
        &self,
        ctx: &SecurityContext,
    ) -> Result<AgentConfigListDto, TradingDashboardError> {
        info!(subject_id = %ctx.subject_id(), "Fetching agent config");
        let conn = self.model_conn()?;
        let scope = AccessScope::default();

        let rows = agent_config::Entity::find()
            .secure()
            .scope_with(&scope)
            .all(&conn)
            .await
            .map_err(db_err)?;

        let configs: Vec<AgentConfigDto> = rows.into_iter().map(|r| AgentConfigDto {
            id: r.id,
            agent_type: r.agent_type,
            enabled: r.enabled.unwrap_or(false),
            mode: r.mode,
            decision_frequency_minutes: r.decision_frequency_minutes,
            confidence_threshold: r.confidence_threshold,
            llm_provider: r.llm_provider,
            llm_model: r.llm_model,
            updated_at: r.updated_at.map(|t| t.format("%Y-%m-%dT%H:%M:%S").to_string()),
        }).collect();

        Ok(AgentConfigListDto { configs })
    }

    // ================================================================
    // Agent Impact — buy-impact, sell-impact, effectiveness
    // ================================================================

    pub async fn get_agent_trade_impacts(
        &self,
        ctx: &SecurityContext,
        action_filter: Option<&str>,
    ) -> Result<AgentTradeImpactListDto, TradingDashboardError> {
        info!(subject_id = %ctx.subject_id(), "Fetching agent trade impacts");
        let conn = self.model_conn()?;
        let scope = AccessScope::default();

        let mut query = agent_trade_impact::Entity::find()
            .order_by_desc(agent_trade_impact::Column::CreatedAt);

        if let Some(action) = action_filter {
            query = query.filter(agent_trade_impact::Column::ActionTaken.eq(action));
        }

        let rows = query
            .secure()
            .scope_with(&scope)
            .all(&conn)
            .await
            .map_err(db_err)?;

        let dec_to_f64 = |d: &sea_orm::prelude::Decimal| -> f64 { d.to_string().parse::<f64>().unwrap_or(0.0) };

        let impacts: Vec<AgentTradeImpactDto> = rows.into_iter().map(|r| AgentTradeImpactDto {
            id: r.id,
            pair: r.pair,
            agent_recommendation: r.agent_recommendation,
            recommendation_confidence: r.recommendation_confidence,
            action_taken: r.action_taken,
            entry_price: r.entry_price.as_ref().map(dec_to_f64),
            exit_price: r.exit_price.as_ref().map(dec_to_f64),
            profit_pct: r.profit_pct,
            counterfactual_profit_pct: r.counterfactual_profit_pct,
            created_at: r.created_at.map(|t| t.format("%Y-%m-%dT%H:%M:%S").to_string()),
        }).collect();

        Ok(AgentTradeImpactListDto { impacts })
    }

    pub async fn get_agent_effectiveness(
        &self,
        ctx: &SecurityContext,
    ) -> Result<AgentEffectivenessDto, TradingDashboardError> {
        info!(subject_id = %ctx.subject_id(), "Fetching agent effectiveness");
        let conn = self.model_conn()?;
        let scope = AccessScope::default();

        let rows = agent_trade_impact::Entity::find()
            .secure()
            .scope_with(&scope)
            .all(&conn)
            .await
            .map_err(db_err)?;

        let total_trades = rows.len() as i64;
        let agent_influenced = rows.iter().filter(|r| r.agent_recommendation.is_some()).count() as i64;

        let with_agent: Vec<f64> = rows.iter()
            .filter(|r| r.agent_recommendation.is_some())
            .filter_map(|r| r.profit_pct)
            .collect();
        let without_agent: Vec<f64> = rows.iter()
            .filter(|r| r.counterfactual_profit_pct.is_some())
            .filter_map(|r| r.counterfactual_profit_pct)
            .collect();

        let avg_profit_with = if with_agent.is_empty() { 0.0 } else { (with_agent.iter().sum::<f64>() / with_agent.len() as f64 * 100.0).round() / 100.0 };
        let avg_profit_without = if without_agent.is_empty() { 0.0 } else { (without_agent.iter().sum::<f64>() / without_agent.len() as f64 * 100.0).round() / 100.0 };

        Ok(AgentEffectivenessDto {
            total_trades,
            agent_influenced,
            avg_profit_with_agent: avg_profit_with,
            avg_profit_without_agent: avg_profit_without,
            agent_value_add: (avg_profit_with - avg_profit_without * 100.0).round() / 100.0,
        })
    }

    // ================================================================
    // Performance — overview, strategy-comparison, alerts
    // ================================================================

    pub async fn get_performance_overview(
        &self,
        ctx: &SecurityContext,
    ) -> Result<PerformanceOverviewDto, TradingDashboardError> {
        info!(subject_id = %ctx.subject_id(), "Fetching performance overview");
        let conn = self.db.conn().map_err(db_err)?;
        let scope = AccessScope::default();

        let all_pnl = pnl::Entity::find()
            .filter(pnl::Column::Pnl.is_not_null())
            .secure()
            .scope_with(&scope)
            .all(&conn)
            .await
            .map_err(db_err)?;

        let total_pnl: f64 = all_pnl.iter().filter_map(|r| r.pnl).sum();
        let total_trades = all_pnl.len() as i64;
        let wins = all_pnl.iter().filter(|r| r.pnl.unwrap_or(0.0) > 0.0).count() as f64;
        let win_rate = if total_trades > 0 { (wins / total_trades as f64 * 1000.0).round() / 10.0 } else { 0.0 };

        let mut day_pnl: HashMap<String, f64> = HashMap::new();
        for r in &all_pnl {
            let date = r.timestamp.map(|t| t.format("%Y-%m-%d").to_string()).unwrap_or_default();
            if date.is_empty() { continue; }
            *day_pnl.entry(date).or_insert(0.0) += r.pnl.unwrap_or(0.0);
        }

        let day_vals: Vec<f64> = day_pnl.values().copied().collect();
        let best_day_pnl = day_vals.iter().cloned().fold(0.0_f64, f64::max);
        let worst_day_pnl = day_vals.iter().cloned().fold(0.0_f64, f64::min);
        let avg_daily_pnl = if day_vals.is_empty() { 0.0 } else { (day_vals.iter().sum::<f64>() / day_vals.len() as f64 * 100.0).round() / 100.0 };

        // Simple max drawdown from daily P&L
        let mut sorted_dates: Vec<(&String, &f64)> = day_pnl.iter().collect();
        sorted_dates.sort_by(|a, b| a.0.cmp(b.0));
        let mut peak = 0.0_f64;
        let mut max_drawdown = 0.0_f64;
        let mut cumulative = 0.0_f64;
        for &(_, &pnl_val) in &sorted_dates {
            cumulative += pnl_val;
            peak = peak.max(cumulative);
            max_drawdown = max_drawdown.min(cumulative - peak);
        }

        // Current streak (consecutive positive days from most recent)
        let mut streak = 0_i64;
        for &(_, &pnl_val) in sorted_dates.iter().rev() {
            if pnl_val > 0.0 { streak += 1; } else { break; }
        }

        Ok(PerformanceOverviewDto {
            total_pnl: (total_pnl * 100.0).round() / 100.0,
            win_rate,
            total_trades,
            best_day_pnl: (best_day_pnl * 100.0).round() / 100.0,
            worst_day_pnl: (worst_day_pnl * 100.0).round() / 100.0,
            avg_daily_pnl,
            max_drawdown: (max_drawdown * 100.0).round() / 100.0,
            current_streak: streak,
        })
    }

    pub async fn get_strategy_comparison(
        &self,
        ctx: &SecurityContext,
    ) -> Result<StrategyComparisonListDto, TradingDashboardError> {
        info!(subject_id = %ctx.subject_id(), "Fetching strategy comparison");
        let conn = self.db.conn().map_err(db_err)?;
        let scope = AccessScope::default();

        let buys = buy_orders::Entity::find()
            .secure()
            .scope_with(&scope)
            .all(&conn)
            .await
            .map_err(db_err)?;

        let mut strategy_map: HashMap<String, (i64, i64, f64)> = HashMap::new();
        for b in &buys {
            let strategy = b.buy_strategy.map(|s| format!("strategy_{s}")).unwrap_or_else(|| "unknown".to_string());
            let entry = strategy_map.entry(strategy).or_insert((0, 0, 0.0));
            entry.0 += 1;
        }

        let strategies: Vec<StrategyComparisonDto> = strategy_map.into_iter().map(|(strategy, (trades, _wins, _pnl))| {
            StrategyComparisonDto {
                strategy,
                trades,
                win_rate: 0.0,
                total_pnl: 0.0,
                avg_pnl: 0.0,
            }
        }).collect();

        Ok(StrategyComparisonListDto { strategies })
    }

    pub async fn get_performance_alerts(
        &self,
        ctx: &SecurityContext,
    ) -> Result<PerformanceAlertsListDto, TradingDashboardError> {
        info!(subject_id = %ctx.subject_id(), "Fetching performance alerts");
        let conn = self.db.conn().map_err(db_err)?;
        let scope = AccessScope::default();

        let mut alerts: Vec<PerformanceAlertDto> = Vec::new();

        // Check recent P&L trend
        let recent_pnl = pnl::Entity::find()
            .filter(pnl::Column::Pnl.is_not_null())
            .order_by_desc(pnl::Column::Timestamp)
            .secure()
            .scope_with(&scope)
            .all(&conn)
            .await
            .map_err(db_err)?;

        let last_10: Vec<f64> = recent_pnl.iter().take(10).filter_map(|r| r.pnl).collect();
        let losing_count = last_10.iter().filter(|&&p| p < 0.0).count();
        if losing_count >= 7 {
            alerts.push(PerformanceAlertDto {
                alert_type: "losing_streak".to_string(),
                message: format!("{losing_count}/10 recent trades are losses"),
                severity: "warning".to_string(),
                timestamp: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
            });
        }

        let pnl_sum: f64 = last_10.iter().sum();
        if pnl_sum < -5.0 {
            alerts.push(PerformanceAlertDto {
                alert_type: "significant_loss".to_string(),
                message: format!("Last 10 trades P&L: {pnl_sum:.2} USDC"),
                severity: "error".to_string(),
                timestamp: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
            });
        }

        Ok(PerformanceAlertsListDto { alerts })
    }

    // ================================================================
    // DB Tables
    // ================================================================

    pub async fn get_db_tables(
        &self,
        ctx: &SecurityContext,
    ) -> Result<DbTablesListDto, TradingDashboardError> {
        info!(subject_id = %ctx.subject_id(), "Fetching DB tables");

        // Return table list with approximate row counts from both databases
        let mut tables: Vec<DbTableDto> = Vec::new();

        // Binance DB tables
        let conn = self.db.conn().map_err(db_err)?;
        let scope = AccessScope::default();

        let t_count = |count: u64| -> DbTableDto { DbTableDto { table_name: String::new(), row_count: count as i64 } };
        let _ = t_count(0); // suppress unused warning

        // Use known entity counts
        let buy_count = buy_orders::Entity::find().secure().scope_with(&scope).count(&conn).await.map_err(db_err)?;
        let sell_count = sell_orders::Entity::find().secure().scope_with(&scope).count(&conn).await.map_err(db_err)?;
        let pnl_count = pnl::Entity::find().secure().scope_with(&scope).count(&conn).await.map_err(db_err)?;

        tables.push(DbTableDto { table_name: "buy_orders".to_string(), row_count: buy_count as i64 });
        tables.push(DbTableDto { table_name: "sell_orders".to_string(), row_count: sell_count as i64 });
        tables.push(DbTableDto { table_name: "pnl".to_string(), row_count: pnl_count as i64 });

        // Model_Data tables (if configured)
        if let Ok(model_conn) = self.model_conn() {
            let ad_count = agent_decisions::Entity::find().secure().scope_with(&scope).count(&model_conn).await.map_err(db_err)?;
            let mth_count = model_training_history::Entity::find().secure().scope_with(&scope).count(&model_conn).await.map_err(db_err)?;
            let ph_count = prediction_history_v2::Entity::find().secure().scope_with(&scope).count(&model_conn).await.map_err(db_err)?;
            let ms_count = market_sentiment::Entity::find().secure().scope_with(&scope).count(&model_conn).await.map_err(db_err)?;

            tables.push(DbTableDto { table_name: "agent_decisions".to_string(), row_count: ad_count as i64 });
            tables.push(DbTableDto { table_name: "model_training_history".to_string(), row_count: mth_count as i64 });
            tables.push(DbTableDto { table_name: "prediction_history_v2".to_string(), row_count: ph_count as i64 });
            tables.push(DbTableDto { table_name: "market_sentiment".to_string(), row_count: ms_count as i64 });
        }

        tables.sort_by(|a, b| b.row_count.cmp(&a.row_count));
        Ok(DbTablesListDto { tables })
    }
}
