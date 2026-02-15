use sea_orm::{ConnectionTrait, DatabaseConnection};
use modkit_security::SecurityContext;

use crate::config::PerformanceMonitorConfig;
use crate::api::rest::dto::*;
use crate::domain::error::DomainError;

fn db_err(e: impl std::fmt::Display) -> DomainError {
    DomainError::Database(e.to_string())
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

fn opt_f64(row: &sea_orm::QueryResult, idx: usize) -> f64 {
    row.try_get_by_index::<f64>(idx).unwrap_or(0.0)
}

fn opt_i64(row: &sea_orm::QueryResult, idx: usize) -> i64 {
    row.try_get_by_index::<i64>(idx).unwrap_or(0)
}

pub struct PerformanceMonitorService {
    prod_db: Option<DatabaseConnection>,
    testnet_db: Option<DatabaseConnection>,
    _config: PerformanceMonitorConfig,
}

impl PerformanceMonitorService {
    pub fn new(
        prod_db: Option<DatabaseConnection>,
        testnet_db: Option<DatabaseConnection>,
        config: PerformanceMonitorConfig,
    ) -> Self {
        Self {
            prod_db,
            testnet_db,
            _config: config,
        }
    }

    fn db_for_env(&self, env: &str) -> Result<&DatabaseConnection, DomainError> {
        match env {
            "testnet" => self.testnet_db.as_ref(),
            _ => self.prod_db.as_ref(),
        }
        .ok_or_else(|| DomainError::Database(format!("No database configured for env '{env}'")))
    }

    // ================================================================
    // GET /performance-monitor/v1/overview
    // ================================================================
    pub async fn get_overview(
        &self,
        _ctx: &SecurityContext,
        env: &str,
        hours: i64,
    ) -> Result<PerformanceOverviewResponse, DomainError> {
        let db = self.db_for_env(env)?;
        let backend = db.get_database_backend();

        // Main overview from pnl table
        let query = format!(
            r#"SELECT
                COUNT(*) as total_trades,
                SUM(CASE WHEN pnl > 0 THEN 1 ELSE 0 END) as winning_trades,
                SUM(CASE WHEN pnl < 0 THEN 1 ELSE 0 END) as losing_trades,
                AVG(pnl) as avg_profit_usdc,
                MAX(pnl) as max_profit_usdc,
                MIN(pnl) as max_loss_usdc,
                SUM(pnl) as total_profit_usdc
            FROM pnl
            WHERE timestamp >= NOW() - INTERVAL '{hours} hours'"#
        );

        let rows = db.query_all(sea_orm::Statement::from_string(backend, query))
            .await
            .map_err(db_err)?;

        let row = match rows.first() {
            Some(r) => r,
            None => return Ok(empty_overview(hours)),
        };

        let total_trades = opt_i64(row, 0);
        if total_trades == 0 {
            return Ok(empty_overview(hours));
        }

        let winning_trades = opt_i64(row, 1);
        let losing_trades = opt_i64(row, 2);
        let avg_profit = opt_f64(row, 3);
        let max_profit = opt_f64(row, 4);
        let max_loss = opt_f64(row, 5);
        let total_profit = opt_f64(row, 6);
        let win_rate = if total_trades > 0 {
            winning_trades as f64 / total_trades as f64 * 100.0
        } else {
            0.0
        };

        // ML vs Signal breakdown
        let strategy_query = format!(
            r#"SELECT
                SUM(CASE WHEN b.buy_strategy IN (4, 7) THEN 1 ELSE 0 END) as ml_trades,
                SUM(CASE WHEN b.buy_strategy IN (6, 8) THEN 1 ELSE 0 END) as signal_trades
            FROM pnl p
            LEFT JOIN buy_orders b ON p.client_order_id = b.client_order_id
            WHERE p.timestamp >= NOW() - INTERVAL '{hours} hours'"#
        );

        let strat_rows = db.query_all(sea_orm::Statement::from_string(backend, strategy_query))
            .await
            .map_err(db_err)?;

        let (ml_trades, signal_trades) = strat_rows.first()
            .map(|r| (opt_i64(r, 0), opt_i64(r, 1)))
            .unwrap_or((0, 0));

        let ml_pct = if total_trades > 0 { ml_trades as f64 / total_trades as f64 * 100.0 } else { 0.0 };
        let signal_pct = if total_trades > 0 { signal_trades as f64 / total_trades as f64 * 100.0 } else { 0.0 };

        // Profit factor
        let win_query = format!(
            r#"SELECT AVG(pnl) as avg_win FROM pnl
            WHERE timestamp >= NOW() - INTERVAL '{hours} hours' AND pnl > 0"#
        );
        let loss_query = format!(
            r#"SELECT AVG(ABS(pnl)) as avg_loss FROM pnl
            WHERE timestamp >= NOW() - INTERVAL '{hours} hours' AND pnl < 0"#
        );

        let avg_win = db.query_all(sea_orm::Statement::from_string(backend, win_query))
            .await.map_err(db_err)?
            .first().map(|r| opt_f64(r, 0).abs()).unwrap_or(0.0);

        let avg_loss = db.query_all(sea_orm::Statement::from_string(backend, loss_query))
            .await.map_err(db_err)?
            .first().map(|r| opt_f64(r, 0)).unwrap_or(1.0);

        let profit_factor = if losing_trades > 0 && avg_loss > 0.0 {
            (avg_win * winning_trades as f64) / (avg_loss * losing_trades as f64)
        } else {
            0.0
        };

        Ok(PerformanceOverviewResponse {
            total_trades,
            winning_trades,
            losing_trades,
            win_rate: round2(win_rate),
            avg_profit: round2(avg_profit),
            max_profit: round2(max_profit),
            max_loss: round2(max_loss),
            profit_factor: round2(profit_factor),
            ml_trades,
            signal_trades,
            ml_percentage: round2(ml_pct),
            signal_percentage: round2(signal_pct),
            total_profit_usdc: round2(total_profit),
            time_period_hours: hours,
        })
    }

    // ================================================================
    // GET /performance-monitor/v1/strategy-comparison
    // ================================================================
    pub async fn get_strategy_comparison(
        &self,
        _ctx: &SecurityContext,
        env: &str,
        hours: i64,
    ) -> Result<StrategyComparisonResponse, DomainError> {
        let db = self.db_for_env(env)?;
        let backend = db.get_database_backend();

        let ml_query = format!(
            r#"SELECT
                COUNT(*) as total_trades,
                SUM(CASE WHEN p.pnl > 0 THEN 1 ELSE 0 END) as winning_trades,
                AVG(p.pnl) as avg_profit,
                SUM(p.pnl) as total_profit_usdc
            FROM pnl p
            LEFT JOIN buy_orders b ON p.client_order_id = b.client_order_id
            WHERE p.timestamp >= NOW() - INTERVAL '{hours} hours'
            AND b.buy_strategy IN (4, 7)"#
        );

        let signal_query = format!(
            r#"SELECT
                COUNT(*) as total_trades,
                SUM(CASE WHEN p.pnl > 0 THEN 1 ELSE 0 END) as winning_trades,
                AVG(p.pnl) as avg_profit,
                SUM(p.pnl) as total_profit_usdc
            FROM pnl p
            LEFT JOIN buy_orders b ON p.client_order_id = b.client_order_id
            WHERE p.timestamp >= NOW() - INTERVAL '{hours} hours'
            AND b.buy_strategy IN (6, 8)"#
        );

        let ml_rows = db.query_all(sea_orm::Statement::from_string(backend, ml_query))
            .await.map_err(db_err)?;
        let signal_rows = db.query_all(sea_orm::Statement::from_string(backend, signal_query))
            .await.map_err(db_err)?;

        let ml_data = parse_strategy_row("ML-Only", ml_rows.first());
        let signal_data = parse_strategy_row("Signal-Based", signal_rows.first());

        let win_rate_delta = round2(ml_data.win_rate - signal_data.win_rate);
        let avg_profit_delta = round2(ml_data.avg_profit - signal_data.avg_profit);
        let better = if win_rate_delta > 0.0 {
            "ML-Only"
        } else if win_rate_delta < 0.0 {
            "Signal-Based"
        } else {
            "Equal"
        };

        Ok(StrategyComparisonResponse {
            ml_strategy: ml_data,
            signal_strategy: signal_data,
            comparison: ComparisonDto {
                win_rate_delta,
                avg_profit_delta,
                better_strategy: better.to_string(),
            },
            time_period_hours: hours,
        })
    }

    // ================================================================
    // GET /performance-monitor/v1/alerts
    // ================================================================
    pub async fn get_alerts(
        &self,
        ctx: &SecurityContext,
        env: &str,
    ) -> Result<AlertsResponse, DomainError> {
        let mut alerts = Vec::new();

        // Check recent 24h win rate
        let overview_24h = self.get_overview(ctx, env, 24).await?;
        if overview_24h.total_trades >= 10 && overview_24h.win_rate < 40.0 {
            alerts.push(AlertDto {
                severity: "high".to_string(),
                alert_type: "low_win_rate".to_string(),
                message: format!("Win rate in last 24h is low: {}%", overview_24h.win_rate),
                value: overview_24h.win_rate,
                threshold: 40.0,
                recommendation: "Review recent trades and market conditions. Consider pausing trading.".to_string(),
            });
        }

        // Check recent losses
        if overview_24h.total_profit_usdc < -50.0 {
            alerts.push(AlertDto {
                severity: "high".to_string(),
                alert_type: "significant_loss".to_string(),
                message: format!("Significant losses in last 24h: ${}", overview_24h.total_profit_usdc),
                value: overview_24h.total_profit_usdc,
                threshold: -50.0,
                recommendation: "Review risk management settings and consider reducing position sizes.".to_string(),
            });
        }

        // Check strategy performance
        let comparison = self.get_strategy_comparison(ctx, env, 24).await?;
        if comparison.ml_strategy.total_trades >= 5
            && comparison.signal_strategy.total_trades >= 5
            && comparison.ml_strategy.win_rate < comparison.signal_strategy.win_rate - 10.0
        {
            alerts.push(AlertDto {
                severity: "medium".to_string(),
                alert_type: "ml_underperformance".to_string(),
                message: format!(
                    "ML strategy underperforming: {}% vs Signal {}%",
                    comparison.ml_strategy.win_rate, comparison.signal_strategy.win_rate
                ),
                value: comparison.ml_strategy.win_rate,
                threshold: comparison.signal_strategy.win_rate,
                recommendation: "ML models may need retraining. Check model age and validation metrics.".to_string(),
            });
        }

        // Check for no trades in last 6 hours
        let overview_6h = self.get_overview(ctx, env, 6).await?;
        if overview_6h.total_trades == 0 {
            alerts.push(AlertDto {
                severity: "low".to_string(),
                alert_type: "no_activity".to_string(),
                message: "No trades executed in last 6 hours".to_string(),
                value: 0.0,
                threshold: 1.0,
                recommendation: "Verify bot is running and market conditions are suitable for trading.".to_string(),
            });
        }

        let count = alerts.len() as i64;
        Ok(AlertsResponse {
            alert_count: count,
            alerts,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }
}

fn empty_overview(hours: i64) -> PerformanceOverviewResponse {
    PerformanceOverviewResponse {
        total_trades: 0,
        winning_trades: 0,
        losing_trades: 0,
        win_rate: 0.0,
        avg_profit: 0.0,
        max_profit: 0.0,
        max_loss: 0.0,
        profit_factor: 0.0,
        ml_trades: 0,
        signal_trades: 0,
        ml_percentage: 0.0,
        signal_percentage: 0.0,
        total_profit_usdc: 0.0,
        time_period_hours: hours,
    }
}

fn parse_strategy_row(name: &str, row: Option<&sea_orm::QueryResult>) -> StrategyDataDto {
    let Some(row) = row else {
        return StrategyDataDto {
            strategy_name: name.to_string(),
            total_trades: 0,
            winning_trades: 0,
            win_rate: 0.0,
            avg_profit: 0.0,
            total_profit_usdc: 0.0,
        };
    };

    let total = opt_i64(row, 0);
    let wins = opt_i64(row, 1);
    let avg = opt_f64(row, 2);
    let profit = opt_f64(row, 3);
    let rate = if total > 0 { wins as f64 / total as f64 * 100.0 } else { 0.0 };

    StrategyDataDto {
        strategy_name: name.to_string(),
        total_trades: total,
        winning_trades: wins,
        win_rate: round2(rate),
        avg_profit: round2(avg),
        total_profit_usdc: round2(profit),
    }
}
