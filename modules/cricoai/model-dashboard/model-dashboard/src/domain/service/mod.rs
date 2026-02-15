use std::collections::HashMap;

use sea_orm::{ConnectionTrait, DatabaseConnection};
use rust_decimal::Decimal;
use modkit_security::SecurityContext;

use crate::config::ModelDashboardConfig;
use crate::api::rest::dto::*;
use crate::domain::error::DomainError;

fn db_err(e: impl std::fmt::Display) -> DomainError {
    DomainError::Database(e.to_string())
}

/// Helper to read an optional Decimal column and convert to f64.
fn dec_opt(row: &sea_orm::QueryResult, idx: usize) -> Option<f64> {
    row.try_get_by_index::<Decimal>(idx)
        .ok()
        .map(|d| d.to_string().parse::<f64>().unwrap_or(0.0))
}

/// Helper to read a count/sum column as i64.
/// PostgreSQL COUNT/SUM returns bigint; ROUND returns numeric.
fn int_val(row: &sea_orm::QueryResult, idx: usize) -> i64 {
    row.try_get_by_index::<i64>(idx)
        .or_else(|_| row.try_get_by_index::<i32>(idx).map(|v| v as i64))
        .or_else(|_| {
            row.try_get_by_index::<Decimal>(idx)
                .map(|d| d.to_string().parse::<i64>().unwrap_or(0))
        })
        .unwrap_or(0)
}

/// Helper to read an optional Decimal column and round to f64.
fn dec_f64_round(row: &sea_orm::QueryResult, idx: usize, places: u32) -> Option<f64> {
    dec_opt(row, idx).map(|v| {
        let factor = 10f64.powi(places as i32);
        (v * factor).round() / factor
    })
}

fn time_filter_sql(time_range: &str, ts_col: &str) -> String {
    match time_range {
        "24h" => format!("AND {ts_col} >= NOW() - INTERVAL '24 hours'"),
        "7d" => format!("AND {ts_col} >= NOW() - INTERVAL '7 days'"),
        "30d" => format!("AND {ts_col} >= NOW() - INTERVAL '30 days'"),
        _ => String::new(), // "all"
    }
}

fn interval_filter_sql(interval: &str) -> String {
    match interval {
        "1m" => "AND interval = '1m'".to_string(),
        "5m" => "AND interval = '5m'".to_string(),
        _ => String::new(), // "all"
    }
}

pub struct ModelDashboardService {
    model_db: DatabaseConnection,
    _config: ModelDashboardConfig,
}

impl ModelDashboardService {
    pub fn new(model_db: DatabaseConnection, config: ModelDashboardConfig) -> Self {
        Self {
            model_db,
            _config: config,
        }
    }

    // ================================================================
    // GET /model-dashboard/v1/summary
    // ================================================================
    pub async fn get_summary(
        &self,
        _ctx: &SecurityContext,
        interval: &str,
        time_range: &str,
    ) -> Result<ModelSummaryResponse, DomainError> {
        let backend = self.model_db.get_database_backend();
        let tf = time_filter_sql(time_range, "trained_at");
        let ivf = interval_filter_sql(interval);

        // Summary by interval
        let query = format!(
            r#"SELECT
                COALESCE(interval, '5m') as interval,
                COUNT(*) as total_trainings,
                SUM(CASE WHEN promoted THEN 1 ELSE 0 END) as promoted_count,
                SUM(CASE WHEN NOT promoted THEN 1 ELSE 0 END) as rejected_count,
                ROUND(AVG(CASE WHEN promoted THEN win_rate END)::numeric, 2) as avg_promoted_win_rate,
                ROUND(AVG(CASE WHEN promoted THEN total_return END)::numeric, 2) as avg_promoted_return,
                ROUND(AVG(CASE WHEN promoted THEN sharpe_ratio END)::numeric, 2) as avg_promoted_sharpe,
                ROUND(AVG(CASE WHEN promoted THEN profit_factor END)::numeric, 2) as avg_promoted_profit_factor,
                SUM(CASE WHEN promoted AND sharpe_ratio IS NOT NULL THEN 1 ELSE 0 END) as promoted_with_sharpe_count,
                SUM(CASE WHEN promoted AND profit_factor IS NOT NULL THEN 1 ELSE 0 END) as promoted_with_profit_factor_count,
                ROUND(AVG(training_duration_seconds)::numeric, 1) as avg_training_duration,
                COUNT(DISTINCT symbol) as unique_symbols
            FROM model_training_history
            WHERE 1=1 {tf} {ivf}
            GROUP BY COALESCE(interval, '5m')
            ORDER BY interval"#
        );

        let rows = self.model_db
            .query_all(sea_orm::Statement::from_string(backend, query))
            .await
            .map_err(db_err)?;

        let mut by_interval = Vec::new();
        for row in &rows {
            let total = int_val(row, 1);
            let promoted = int_val(row, 2);
            let rate = if total > 0 {
                (promoted as f64 / total as f64 * 100.0 * 10.0).round() / 10.0
            } else {
                0.0
            };

            by_interval.push(IntervalSummaryDto {
                interval: row.try_get_by_index::<String>(0).unwrap_or_default(),
                total_trainings: total,
                promoted_count: promoted,
                rejected_count: int_val(row, 3),
                avg_promoted_win_rate: dec_opt(row, 4),
                avg_promoted_return: dec_opt(row, 5),
                avg_promoted_sharpe: dec_opt(row, 6),
                avg_promoted_profit_factor: dec_opt(row, 7),
                promoted_with_sharpe_count: int_val(row, 8),
                promoted_with_profit_factor_count: int_val(row, 9),
                avg_training_duration: dec_opt(row, 10),
                unique_symbols: int_val(row, 11),
                promotion_rate: rate,
            });
        }

        // Overall totals
        let totals_sql = format!(
            r#"SELECT
                COUNT(*) as total,
                SUM(CASE WHEN promoted THEN 1 ELSE 0 END) as promoted,
                COUNT(DISTINCT symbol) as symbols,
                SUM(CASE WHEN promoted AND sharpe_ratio IS NOT NULL THEN 1 ELSE 0 END) as promoted_with_sharpe,
                SUM(CASE WHEN promoted AND profit_factor IS NOT NULL THEN 1 ELSE 0 END) as promoted_with_pf
            FROM model_training_history
            WHERE 1=1 {tf} {ivf}"#
        );

        let totals_rows = self.model_db
            .query_all(sea_orm::Statement::from_string(backend, totals_sql))
            .await
            .map_err(db_err)?;

        let totals = if let Some(row) = totals_rows.first() {
            let total = int_val(row, 0);
            let promoted = int_val(row, 1);
            let rate = if total > 0 {
                (promoted as f64 / total as f64 * 100.0 * 10.0).round() / 10.0
            } else {
                0.0
            };
            SummaryTotalsDto {
                total_trainings: total,
                total_promoted: promoted,
                unique_symbols: int_val(row, 2),
                promoted_with_sharpe_count: int_val(row, 3),
                promoted_with_profit_factor_count: int_val(row, 4),
                overall_promotion_rate: rate,
            }
        } else {
            SummaryTotalsDto {
                total_trainings: 0,
                total_promoted: 0,
                unique_symbols: 0,
                promoted_with_sharpe_count: 0,
                promoted_with_profit_factor_count: 0,
                overall_promotion_rate: 0.0,
            }
        };

        Ok(ModelSummaryResponse { by_interval, totals })
    }

    // ================================================================
    // GET /model-dashboard/v1/training-history
    // ================================================================
    pub async fn get_training_history(
        &self,
        _ctx: &SecurityContext,
        interval: &str,
        time_range: &str,
        symbol: Option<&str>,
        promoted_only: bool,
        rejected_only: bool,
        limit: i64,
        offset: i64,
    ) -> Result<TrainingHistoryResponse, DomainError> {
        let backend = self.model_db.get_database_backend();
        let tf = time_filter_sql(time_range, "trained_at");
        let ivf = interval_filter_sql(interval);

        let mut extra = String::new();
        if let Some(sym) = symbol {
            extra.push_str(&format!("AND symbol = '{}'", sym.replace('\'', "''")));
        }
        if promoted_only {
            extra.push_str(" AND promoted = true");
        } else if rejected_only {
            extra.push_str(" AND promoted = false");
        }

        let query = format!(
            r#"SELECT
                id, symbol, COALESCE(interval, '5m') as interval,
                trained_at, promoted, win_rate, total_trades, total_return,
                sharpe_ratio, max_drawdown, profit_factor,
                training_duration_seconds, backtest_duration_seconds,
                rejection_reason,
                wf_median_win_rate, wf_mean_return, wf_max_drawdown_worst, wf_passed_gates,
                oos_win_rate, oos_total_trades, oos_return, oos_max_drawdown, oos_passed_gates
            FROM model_training_history
            WHERE 1=1 {tf} {ivf} {extra}
            ORDER BY trained_at DESC
            LIMIT {limit} OFFSET {offset}"#
        );

        let rows = self.model_db
            .query_all(sea_orm::Statement::from_string(backend, query))
            .await
            .map_err(db_err)?;

        let history: Vec<TrainingHistoryItemDto> = rows.iter().map(|row| {
            TrainingHistoryItemDto {
                id: row.try_get_by_index::<i32>(0).unwrap_or(0),
                symbol: row.try_get_by_index::<String>(1).unwrap_or_default(),
                interval: row.try_get_by_index::<String>(2).unwrap_or_default(),
                trained_at: row.try_get_by_index::<chrono::NaiveDateTime>(3)
                    .map(|d| d.format("%Y-%m-%dT%H:%M:%S").to_string())
                    .unwrap_or_default(),
                promoted: row.try_get_by_index::<bool>(4).unwrap_or(false),
                win_rate: dec_opt(row, 5),
                total_trades: row.try_get_by_index::<i32>(6).ok(),
                total_return: dec_opt(row, 7),
                sharpe_ratio: dec_opt(row, 8),
                max_drawdown: dec_opt(row, 9),
                profit_factor: dec_opt(row, 10),
                training_duration_seconds: dec_opt(row, 11),
                backtest_duration_seconds: dec_opt(row, 12),
                rejection_reason: row.try_get_by_index::<String>(13).ok(),
                wf_median_win_rate: dec_opt(row, 14),
                wf_mean_return: dec_opt(row, 15),
                wf_max_drawdown_worst: dec_opt(row, 16),
                wf_passed_gates: row.try_get_by_index::<bool>(17).ok(),
                oos_win_rate: dec_opt(row, 18),
                oos_total_trades: row.try_get_by_index::<i32>(19).ok(),
                oos_return: dec_opt(row, 20),
                oos_max_drawdown: dec_opt(row, 21),
                oos_passed_gates: row.try_get_by_index::<bool>(22).ok(),
            }
        }).collect();

        // Get total count
        let count_sql = format!(
            r#"SELECT COUNT(*) as total
            FROM model_training_history
            WHERE 1=1 {tf} {ivf} {extra}"#
        );

        let count_rows = self.model_db
            .query_all(sea_orm::Statement::from_string(backend, count_sql))
            .await
            .map_err(db_err)?;

        let total = count_rows.first()
            .map(|r| int_val(r, 0))
            .unwrap_or(0);

        Ok(TrainingHistoryResponse {
            history,
            total,
            limit,
            offset,
        })
    }

    // ================================================================
    // GET /model-dashboard/v1/rejection-reasons
    // ================================================================
    pub async fn get_rejection_reasons(
        &self,
        _ctx: &SecurityContext,
        interval: &str,
        time_range: &str,
    ) -> Result<RejectionReasonsResponse, DomainError> {
        let backend = self.model_db.get_database_backend();
        let tf = time_filter_sql(time_range, "trained_at");
        let ivf = interval_filter_sql(interval);

        let query = format!(
            r#"SELECT
                COALESCE(interval, '5m') as interval,
                COALESCE(rejection_reason, 'Unknown') as reason,
                COUNT(*) as count
            FROM model_training_history
            WHERE promoted = false {tf} {ivf}
            GROUP BY COALESCE(interval, '5m'), COALESCE(rejection_reason, 'Unknown')
            ORDER BY interval, count DESC"#
        );

        let rows = self.model_db
            .query_all(sea_orm::Statement::from_string(backend, query))
            .await
            .map_err(db_err)?;

        let mut by_interval: HashMap<String, Vec<RejectionReasonDto>> = HashMap::new();
        for row in &rows {
            let interval_key = row.try_get_by_index::<String>(0).unwrap_or_default();
            let reason = row.try_get_by_index::<String>(1).unwrap_or_default();
            let count = int_val(row, 2);

            by_interval
                .entry(interval_key)
                .or_default()
                .push(RejectionReasonDto { reason, count });
        }

        Ok(RejectionReasonsResponse { by_interval })
    }

    // ================================================================
    // GET /model-dashboard/v1/symbol-status
    // ================================================================
    pub async fn get_symbol_status(
        &self,
        _ctx: &SecurityContext,
        interval: &str,
    ) -> Result<SymbolStatusResponse, DomainError> {
        let backend = self.model_db.get_database_backend();
        let ivf = interval_filter_sql(interval);

        let query = format!(
            r#"WITH latest_training AS (
                SELECT DISTINCT ON (symbol, COALESCE(interval, '5m'))
                    symbol,
                    COALESCE(interval, '5m') as interval,
                    trained_at,
                    promoted,
                    win_rate,
                    total_return,
                    sharpe_ratio,
                    rejection_reason,
                    EXTRACT(EPOCH FROM (NOW() - trained_at)) / 86400.0 as model_age_days
                FROM model_training_history
                WHERE 1=1 {ivf}
                ORDER BY symbol, COALESCE(interval, '5m'), trained_at DESC
            )
            SELECT * FROM latest_training
            ORDER BY interval, symbol"#
        );

        let rows = self.model_db
            .query_all(sea_orm::Statement::from_string(backend, query))
            .await
            .map_err(db_err)?;

        let symbols: Vec<SymbolStatusDto> = rows.iter().map(|row| {
            SymbolStatusDto {
                symbol: row.try_get_by_index::<String>(0).unwrap_or_default(),
                interval: row.try_get_by_index::<String>(1).unwrap_or_default(),
                trained_at: row.try_get_by_index::<chrono::NaiveDateTime>(2)
                    .map(|d| d.format("%Y-%m-%dT%H:%M:%S").to_string())
                    .unwrap_or_default(),
                promoted: row.try_get_by_index::<bool>(3).unwrap_or(false),
                win_rate: dec_f64_round(row, 4, 2),
                total_return: dec_f64_round(row, 5, 2),
                sharpe_ratio: dec_f64_round(row, 6, 2),
                rejection_reason: row.try_get_by_index::<String>(7).ok(),
                model_age_days: dec_f64_round(row, 8, 2),
            }
        }).collect();

        Ok(SymbolStatusResponse { symbols })
    }

    // ================================================================
    // GET /model-dashboard/v1/training-timeline
    // ================================================================
    pub async fn get_training_timeline(
        &self,
        _ctx: &SecurityContext,
        interval: &str,
        time_range: &str,
    ) -> Result<TrainingTimelineResponse, DomainError> {
        let backend = self.model_db.get_database_backend();
        let tf = time_filter_sql(time_range, "trained_at");
        let ivf = interval_filter_sql(interval);

        let query = format!(
            r#"SELECT
                DATE(trained_at) as date,
                COALESCE(interval, '5m') as interval,
                COUNT(*) as total,
                SUM(CASE WHEN promoted THEN 1 ELSE 0 END) as promoted,
                ROUND(AVG(win_rate)::numeric, 2) as avg_win_rate,
                ROUND(AVG(total_return)::numeric, 2) as avg_return
            FROM model_training_history
            WHERE 1=1 {tf} {ivf}
            GROUP BY DATE(trained_at), COALESCE(interval, '5m')
            ORDER BY date DESC, interval"#
        );

        let rows = self.model_db
            .query_all(sea_orm::Statement::from_string(backend, query))
            .await
            .map_err(db_err)?;

        let timeline: Vec<TimelinePointDto> = rows.iter().map(|row| {
            let total = int_val(row, 2);
            let promoted = int_val(row, 3);
            let rate = if total > 0 {
                (promoted as f64 / total as f64 * 100.0 * 10.0).round() / 10.0
            } else {
                0.0
            };

            TimelinePointDto {
                date: row.try_get_by_index::<chrono::NaiveDate>(0)
                    .map(|d| d.format("%Y-%m-%d").to_string())
                    .unwrap_or_default(),
                interval: row.try_get_by_index::<String>(1).unwrap_or_default(),
                total,
                promoted,
                avg_win_rate: dec_opt(row, 4),
                avg_return: dec_opt(row, 5),
                promotion_rate: rate,
            }
        }).collect();

        Ok(TrainingTimelineResponse { timeline })
    }

    // ================================================================
    // GET /model-dashboard/v1/performance-distribution
    // ================================================================
    pub async fn get_performance_distribution(
        &self,
        _ctx: &SecurityContext,
        interval: &str,
        time_range: &str,
        promoted_only: bool,
    ) -> Result<PerformanceDistributionResponse, DomainError> {
        let backend = self.model_db.get_database_backend();
        let tf = time_filter_sql(time_range, "trained_at");
        let ivf = interval_filter_sql(interval);
        let pf = if promoted_only { "AND promoted = true" } else { "" };

        let query = format!(
            r#"SELECT
                COALESCE(interval, '5m') as interval,
                win_rate, total_return, sharpe_ratio, profit_factor, max_drawdown
            FROM model_training_history
            WHERE win_rate IS NOT NULL
                AND total_return IS NOT NULL
                {tf} {ivf} {pf}
            ORDER BY interval, trained_at DESC
            LIMIT 1000"#
        );

        let rows = self.model_db
            .query_all(sea_orm::Statement::from_string(backend, query))
            .await
            .map_err(db_err)?;

        let mut distributions: HashMap<String, Vec<PerformancePointDto>> = HashMap::new();
        distributions.insert("1m".to_string(), Vec::new());
        distributions.insert("5m".to_string(), Vec::new());

        for row in &rows {
            let interval_key = row.try_get_by_index::<String>(0).unwrap_or_default();
            if let Some(list) = distributions.get_mut(&interval_key) {
                list.push(PerformancePointDto {
                    win_rate: dec_opt(row, 1),
                    total_return: dec_opt(row, 2),
                    sharpe_ratio: dec_opt(row, 3),
                    profit_factor: dec_opt(row, 4),
                    max_drawdown: dec_opt(row, 5),
                });
            }
        }

        Ok(PerformanceDistributionResponse { distributions })
    }

    // ================================================================
    // GET /model-dashboard/v1/calibration-metrics
    // ================================================================
    pub async fn get_calibration_metrics(
        &self,
        _ctx: &SecurityContext,
        time_range: &str,
    ) -> Result<CalibrationMetricsResponse, DomainError> {
        let backend = self.model_db.get_database_backend();
        let tf = time_filter_sql(time_range, "verification_timestamp");

        let query = format!(
            r#"SELECT
                symbol,
                model_version,
                AVG(calibration_error) as avg_calibration_error,
                COUNT(*) as predictions,
                SUM(CASE WHEN direction_correct THEN 1 ELSE 0 END)::float / COUNT(*) as accuracy,
                AVG(confidence_score) as avg_confidence,
                AVG(CASE WHEN direction_correct THEN confidence_score END) as avg_confidence_when_correct,
                AVG(CASE WHEN NOT direction_correct THEN confidence_score END) as avg_confidence_when_wrong
            FROM model_calibration_metrics
            WHERE verification_timestamp IS NOT NULL {tf}
            GROUP BY symbol, model_version
            ORDER BY avg_calibration_error DESC
            LIMIT 50"#
        );

        let rows = self.model_db
            .query_all(sea_orm::Statement::from_string(backend, query))
            .await
            .map_err(db_err)?;

        let metrics: Vec<CalibrationMetricDto> = rows.iter().map(|row| {
            CalibrationMetricDto {
                symbol: row.try_get_by_index::<String>(0).unwrap_or_default(),
                model_version: row.try_get_by_index::<String>(1).unwrap_or_default(),
                avg_calibration_error: dec_f64_round(row, 2, 4),
                predictions: int_val(row, 3),
                accuracy: row.try_get_by_index::<f64>(4).ok().map(|v| (v * 10000.0).round() / 10000.0),
                avg_confidence: dec_f64_round(row, 5, 4),
                avg_confidence_when_correct: dec_f64_round(row, 6, 4),
                avg_confidence_when_wrong: dec_f64_round(row, 7, 4),
            }
        }).collect();

        Ok(CalibrationMetricsResponse { calibration_metrics: metrics })
    }

    // ================================================================
    // GET /model-dashboard/v1/regime-analysis
    // ================================================================
    pub async fn get_regime_analysis(
        &self,
        _ctx: &SecurityContext,
        time_range: &str,
    ) -> Result<RegimeAnalysisResponse, DomainError> {
        let backend = self.model_db.get_database_backend();
        let tf = time_filter_sql(time_range, "verification_timestamp");

        let query = format!(
            r#"SELECT
                market_regime,
                regime_conflict,
                COUNT(*) as count,
                AVG(direction_correct::int) as accuracy,
                AVG(confidence_score) as avg_confidence,
                AVG(calibration_error) as avg_calibration_error,
                AVG(regime_penalty) as avg_regime_penalty
            FROM model_calibration_metrics
            WHERE verification_timestamp IS NOT NULL
                AND market_regime IS NOT NULL {tf}
            GROUP BY market_regime, regime_conflict
            ORDER BY market_regime, regime_conflict"#
        );

        let rows = self.model_db
            .query_all(sea_orm::Statement::from_string(backend, query))
            .await
            .map_err(db_err)?;

        let analysis: Vec<RegimeAnalysisDto> = rows.iter().map(|row| {
            RegimeAnalysisDto {
                market_regime: row.try_get_by_index::<String>(0).unwrap_or_default(),
                regime_conflict: row.try_get_by_index::<bool>(1).unwrap_or(false),
                count: int_val(row, 2),
                accuracy: dec_f64_round(row, 3, 4),
                avg_confidence: dec_f64_round(row, 4, 4),
                avg_calibration_error: dec_f64_round(row, 5, 4),
                avg_regime_penalty: dec_f64_round(row, 6, 4),
            }
        }).collect();

        Ok(RegimeAnalysisResponse { regime_analysis: analysis })
    }

    // ================================================================
    // GET /model-dashboard/v1/model-age-impact
    // ================================================================
    pub async fn get_model_age_impact(
        &self,
        _ctx: &SecurityContext,
        time_range: &str,
    ) -> Result<AgeImpactResponse, DomainError> {
        let backend = self.model_db.get_database_backend();
        let tf = time_filter_sql(time_range, "verification_timestamp");

        let query = format!(
            r#"SELECT
                CASE
                    WHEN model_age_days <= 7 THEN '0-7 days'
                    WHEN model_age_days <= 14 THEN '8-14 days'
                    WHEN model_age_days <= 30 THEN '15-30 days'
                    ELSE '30+ days'
                END as age_bucket,
                COUNT(*) as predictions,
                AVG(direction_correct::int) as accuracy,
                AVG(calibration_error) as avg_calibration_error,
                AVG(confidence_score) as avg_confidence,
                AVG(age_penalty) as avg_age_penalty,
                MIN(model_age_days) as min_age,
                MAX(model_age_days) as max_age
            FROM model_calibration_metrics
            WHERE verification_timestamp IS NOT NULL
                AND model_age_days IS NOT NULL {tf}
            GROUP BY age_bucket
            ORDER BY MIN(model_age_days)"#
        );

        let rows = self.model_db
            .query_all(sea_orm::Statement::from_string(backend, query))
            .await
            .map_err(db_err)?;

        let age_impact: Vec<AgeImpactDto> = rows.iter().map(|row| {
            AgeImpactDto {
                age_bucket: row.try_get_by_index::<String>(0).unwrap_or_default(),
                predictions: int_val(row, 1),
                accuracy: dec_f64_round(row, 2, 4),
                avg_calibration_error: dec_f64_round(row, 3, 4),
                avg_confidence: dec_f64_round(row, 4, 4),
                avg_age_penalty: dec_f64_round(row, 5, 4),
                min_age: dec_f64_round(row, 6, 2),
                max_age: dec_f64_round(row, 7, 2),
            }
        }).collect();

        Ok(AgeImpactResponse { age_impact })
    }
}
