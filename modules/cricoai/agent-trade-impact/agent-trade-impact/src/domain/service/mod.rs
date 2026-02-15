use sea_orm::{ConnectionTrait, DatabaseConnection};
use rust_decimal::Decimal;
use modkit_security::SecurityContext;

use crate::config::AgentTradeImpactConfig;
use crate::api::rest::dto::*;
use crate::domain::error::DomainError;

fn db_err(e: impl std::fmt::Display) -> DomainError {
    DomainError::Database(e.to_string())
}

fn opt_f64(row: &sea_orm::QueryResult, idx: usize) -> Option<f64> {
    row.try_get_by_index::<f64>(idx)
        .ok()
        .or_else(|| {
            row.try_get_by_index::<Decimal>(idx)
                .ok()
                .map(|d| d.to_string().parse::<f64>().unwrap_or(0.0))
        })
}

fn opt_i64(row: &sea_orm::QueryResult, idx: usize) -> i64 {
    row.try_get_by_index::<i64>(idx)
        .or_else(|_| row.try_get_by_index::<i32>(idx).map(|v| v as i64))
        .or_else(|_| {
            row.try_get_by_index::<Decimal>(idx)
                .map(|d| d.to_string().parse::<i64>().unwrap_or(0))
        })
        .unwrap_or(0)
}

fn opt_str(row: &sea_orm::QueryResult, idx: usize) -> Option<String> {
    row.try_get_by_index::<String>(idx).ok()
}

fn opt_ts(row: &sea_orm::QueryResult, idx: usize) -> Option<String> {
    row.try_get_by_index::<chrono::NaiveDateTime>(idx)
        .ok()
        .map(|dt| format!("{}Z", dt.format("%Y-%m-%dT%H:%M:%S")))
}

pub struct AgentTradeImpactService {
    model_db: DatabaseConnection,
    _config: AgentTradeImpactConfig,
}

impl AgentTradeImpactService {
    pub fn new(model_db: DatabaseConnection, config: AgentTradeImpactConfig) -> Self {
        Self { model_db, _config: config }
    }

    // ================================================================
    // GET /agent-trade-impact/v1/buy-impact
    // ================================================================
    pub async fn get_buy_impact(
        &self,
        _ctx: &SecurityContext,
        env: &str,
        days: i64,
        pair: Option<&str>,
    ) -> Result<BuyImpactResponse, DomainError> {
        let backend = self.model_db.get_database_backend();

        let pair_filter = pair
            .map(|p| format!(" AND pair = '{}'", p.replace('\'', "''")))
            .unwrap_or_default();

        let query = format!(
            r#"SELECT
                pair,
                COUNT(*) as total_impacts,
                AVG(recommendation_confidence) as avg_confidence,
                COUNT(CASE WHEN action_taken = 'buy_executed' THEN 1 END) as buys_executed,
                COUNT(CASE WHEN action_taken = 'buy_blocked' THEN 1 END) as buys_blocked,
                COUNT(CASE WHEN action_taken LIKE 'buy_modified%%' THEN 1 END) as buys_modified,
                AVG(CASE WHEN profit_pct IS NOT NULL THEN profit_pct END) as avg_profit_pct,
                AVG(CASE WHEN counterfactual_profit_pct IS NOT NULL THEN counterfactual_profit_pct END) as avg_counterfactual_profit
            FROM agent_trade_impact
            WHERE environment = '{env}'
            AND created_at >= NOW() - INTERVAL '{days} days'
            AND action_taken IN ('buy_executed', 'buy_modified_reduced', 'buy_modified_increased', 'buy_blocked')
            {pair_filter}
            GROUP BY pair
            ORDER BY total_impacts DESC"#
        );

        let rows = self.model_db
            .query_all(sea_orm::Statement::from_string(backend, query))
            .await
            .map_err(db_err)?;

        let buy_impacts: Vec<BuyImpactDto> = rows.iter().map(|row| {
            BuyImpactDto {
                pair: opt_str(row, 0).unwrap_or_default(),
                total_impacts: opt_i64(row, 1),
                avg_confidence: opt_f64(row, 2),
                buys_executed: opt_i64(row, 3),
                buys_blocked: opt_i64(row, 4),
                buys_modified: opt_i64(row, 5),
                avg_profit_pct: opt_f64(row, 6),
                avg_counterfactual_profit: opt_f64(row, 7),
            }
        }).collect();

        Ok(BuyImpactResponse {
            environment: env.to_string(),
            period_days: days,
            buy_impacts,
        })
    }

    // ================================================================
    // GET /agent-trade-impact/v1/sell-impact
    // ================================================================
    pub async fn get_sell_impact(
        &self,
        _ctx: &SecurityContext,
        env: &str,
        days: i64,
        pair: Option<&str>,
    ) -> Result<SellImpactResponse, DomainError> {
        let backend = self.model_db.get_database_backend();

        let pair_filter = pair
            .map(|p| format!(" AND pair = '{}'", p.replace('\'', "''")))
            .unwrap_or_default();

        let query = format!(
            r#"SELECT
                pair,
                COUNT(*) as total_agent_exits,
                AVG(recommendation_confidence) as avg_confidence,
                AVG(profit_pct) as avg_actual_profit,
                AVG(counterfactual_profit_pct) as avg_counterfactual_profit,
                AVG(profit_pct - counterfactual_profit_pct) as avg_agent_value_add,
                COUNT(CASE WHEN profit_pct > 0 THEN 1 END) as profitable_exits,
                COUNT(CASE WHEN profit_pct < 0 THEN 1 END) as loss_exits,
                MAX(created_at) as last_agent_exit
            FROM agent_trade_impact
            WHERE environment = '{env}'
            AND created_at >= NOW() - INTERVAL '{days} days'
            AND action_taken = 'agent_guided_sell'
            {pair_filter}
            GROUP BY pair
            ORDER BY total_agent_exits DESC"#
        );

        let rows = self.model_db
            .query_all(sea_orm::Statement::from_string(backend, query))
            .await
            .map_err(db_err)?;

        let sell_impacts: Vec<SellImpactDto> = rows.iter().map(|row| {
            SellImpactDto {
                pair: opt_str(row, 0).unwrap_or_default(),
                total_agent_exits: opt_i64(row, 1),
                avg_confidence: opt_f64(row, 2),
                avg_actual_profit: opt_f64(row, 3),
                avg_counterfactual_profit: opt_f64(row, 4),
                avg_agent_value_add: opt_f64(row, 5),
                profitable_exits: opt_i64(row, 6),
                loss_exits: opt_i64(row, 7),
                last_agent_exit: opt_ts(row, 8),
            }
        }).collect();

        Ok(SellImpactResponse {
            environment: env.to_string(),
            period_days: days,
            sell_impacts,
        })
    }

    // ================================================================
    // GET /agent-trade-impact/v1/effectiveness
    // ================================================================
    pub async fn get_effectiveness(
        &self,
        _ctx: &SecurityContext,
        env: &str,
        days: i64,
    ) -> Result<EffectivenessResponse, DomainError> {
        let backend = self.model_db.get_database_backend();

        // Overall effectiveness summary
        let summary_query = format!(
            r#"SELECT
                COUNT(*) as total_interventions,
                COUNT(DISTINCT pair) as pairs_affected,
                AVG(recommendation_confidence) as avg_confidence,
                SUM(CASE WHEN action_taken LIKE 'buy%%' THEN 1 ELSE 0 END) as buy_interventions,
                SUM(CASE WHEN action_taken = 'agent_guided_sell' THEN 1 ELSE 0 END) as sell_interventions,
                AVG(CASE WHEN profit_pct IS NOT NULL THEN profit_pct END) as avg_actual_profit,
                AVG(CASE WHEN counterfactual_profit_pct IS NOT NULL THEN counterfactual_profit_pct END) as avg_counterfactual_profit,
                AVG(CASE
                    WHEN profit_pct IS NOT NULL AND counterfactual_profit_pct IS NOT NULL
                    THEN profit_pct - counterfactual_profit_pct
                END) as avg_value_added,
                COUNT(CASE
                    WHEN profit_pct IS NOT NULL AND counterfactual_profit_pct IS NOT NULL
                    AND profit_pct > counterfactual_profit_pct
                    THEN 1
                END) as times_agent_helped,
                COUNT(CASE
                    WHEN profit_pct IS NOT NULL AND counterfactual_profit_pct IS NOT NULL
                    AND profit_pct < counterfactual_profit_pct
                    THEN 1
                END) as times_agent_hurt
            FROM agent_trade_impact
            WHERE environment = '{env}'
            AND created_at >= NOW() - INTERVAL '{days} days'"#
        );

        let sum_rows = self.model_db
            .query_all(sea_orm::Statement::from_string(backend, summary_query))
            .await
            .map_err(db_err)?;

        let summary = sum_rows.first().map(|row| {
            EffectivenessSummaryDto {
                total_interventions: opt_i64(row, 0),
                pairs_affected: opt_i64(row, 1),
                avg_confidence: opt_f64(row, 2),
                buy_interventions: opt_i64(row, 3),
                sell_interventions: opt_i64(row, 4),
                avg_actual_profit: opt_f64(row, 5),
                avg_counterfactual_profit: opt_f64(row, 6),
                avg_value_added: opt_f64(row, 7),
                times_agent_helped: opt_i64(row, 8),
                times_agent_hurt: opt_i64(row, 9),
            }
        }).unwrap_or(EffectivenessSummaryDto {
            total_interventions: 0,
            pairs_affected: 0,
            avg_confidence: None,
            buy_interventions: 0,
            sell_interventions: 0,
            avg_actual_profit: None,
            avg_counterfactual_profit: None,
            avg_value_added: None,
            times_agent_helped: 0,
            times_agent_hurt: 0,
        });

        // Breakdown by action type
        let action_query = format!(
            r#"SELECT
                action_taken,
                COUNT(*) as count,
                AVG(profit_pct) as avg_profit,
                AVG(counterfactual_profit_pct) as avg_counterfactual
            FROM agent_trade_impact
            WHERE environment = '{env}'
            AND created_at >= NOW() - INTERVAL '{days} days'
            GROUP BY action_taken
            ORDER BY count DESC"#
        );

        let action_rows = self.model_db
            .query_all(sea_orm::Statement::from_string(backend, action_query))
            .await
            .map_err(db_err)?;

        let by_action: Vec<ActionBreakdownDto> = action_rows.iter().map(|row| {
            ActionBreakdownDto {
                action_taken: opt_str(row, 0).unwrap_or_default(),
                count: opt_i64(row, 1),
                avg_profit: opt_f64(row, 2),
                avg_counterfactual: opt_f64(row, 3),
            }
        }).collect();

        Ok(EffectivenessResponse {
            environment: env.to_string(),
            period_days: days,
            summary,
            by_action,
        })
    }
}
