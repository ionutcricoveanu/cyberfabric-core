use sea_orm::{ConnectionTrait, DatabaseConnection};
use rust_decimal::Decimal;
use modkit_security::SecurityContext;

use crate::config::AgentAnalyticsConfig;
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

fn opt_bool(row: &sea_orm::QueryResult, idx: usize) -> bool {
    row.try_get_by_index::<bool>(idx).unwrap_or(false)
}

fn opt_ts(row: &sea_orm::QueryResult, idx: usize) -> Option<String> {
    row.try_get_by_index::<chrono::NaiveDateTime>(idx)
        .ok()
        .map(|dt| format!("{}Z", dt.format("%Y-%m-%dT%H:%M:%S")))
}

fn opt_date(row: &sea_orm::QueryResult, idx: usize) -> String {
    row.try_get_by_index::<chrono::NaiveDate>(idx)
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_default()
}

fn opt_json(row: &sea_orm::QueryResult, idx: usize) -> Option<serde_json::Value> {
    row.try_get_by_index::<serde_json::Value>(idx).ok()
}

fn opt_i32(row: &sea_orm::QueryResult, idx: usize) -> Option<i32> {
    row.try_get_by_index::<i32>(idx).ok()
}

pub struct AgentAnalyticsService {
    model_db: DatabaseConnection,
    _config: AgentAnalyticsConfig,
}

impl AgentAnalyticsService {
    pub fn new(model_db: DatabaseConnection, config: AgentAnalyticsConfig) -> Self {
        Self { model_db, _config: config }
    }

    // ================================================================
    // GET /agent-analytics/v1/overview
    // ================================================================
    pub async fn get_overview(
        &self,
        _ctx: &SecurityContext,
    ) -> Result<AgentOverviewResponse, DomainError> {
        let backend = self.model_db.get_database_backend();

        // Agent performance summary
        let agents_rows = self.model_db
            .query_all(sea_orm::Statement::from_string(backend, r#"
                SELECT
                    agent_type, enabled, mode, llm_provider, llm_model,
                    decisions_today, avg_confidence_today, cost_today,
                    positive_outcomes_today, success_rate_today
                FROM agent_performance_summary
            "#.to_string()))
            .await
            .map_err(db_err)?;

        let agents: Vec<AgentInfoDto> = agents_rows.iter().map(|row| {
            AgentInfoDto {
                agent_type: opt_str(row, 0).unwrap_or_default(),
                enabled: opt_bool(row, 1),
                mode: opt_str(row, 2),
                llm_provider: opt_str(row, 3),
                llm_model: opt_str(row, 4),
                decisions_today: opt_i64(row, 5),
                avg_confidence_today: opt_f64(row, 6),
                cost_today: opt_f64(row, 7),
                positive_outcomes_today: opt_i64(row, 8),
                success_rate_today: opt_f64(row, 9),
            }
        }).collect();

        // Overall summary
        let sum_rows = self.model_db
            .query_all(sea_orm::Statement::from_string(backend, r#"
                SELECT
                    COUNT(*) as total_decisions_today,
                    SUM(CASE WHEN status = 'pending' THEN 1 ELSE 0 END) as decisions_pending,
                    SUM(CASE WHEN status = 'applied' THEN 1 ELSE 0 END) as decisions_applied,
                    SUM(CASE WHEN status = 'rejected' THEN 1 ELSE 0 END) as decisions_rejected,
                    AVG(confidence) as avg_confidence,
                    SUM(cost_usd) as total_cost_today
                FROM agent_decisions
                WHERE created_at >= CURRENT_DATE
            "#.to_string()))
            .await
            .map_err(db_err)?;

        let summary = sum_rows.first().map(|row| {
            AgentSummaryDto {
                total_decisions_today: opt_i64(row, 0),
                decisions_pending: opt_i64(row, 1),
                decisions_applied: opt_i64(row, 2),
                decisions_rejected: opt_i64(row, 3),
                avg_confidence: opt_f64(row, 4),
                total_cost_today: opt_f64(row, 5),
            }
        }).unwrap_or(AgentSummaryDto {
            total_decisions_today: 0,
            decisions_pending: 0,
            decisions_applied: 0,
            decisions_rejected: 0,
            avg_confidence: None,
            total_cost_today: None,
        });

        Ok(AgentOverviewResponse { summary, agents })
    }

    // ================================================================
    // GET /agent-analytics/v1/decisions/recent
    // ================================================================
    pub async fn get_recent_decisions(
        &self,
        _ctx: &SecurityContext,
        agent_type: Option<&str>,
        limit: i64,
        hours: i64,
    ) -> Result<RecentDecisionsResponse, DomainError> {
        let backend = self.model_db.get_database_backend();

        let agent_filter = agent_type
            .map(|a| format!(" AND agent_type = '{}'", a.replace('\'', "''")))
            .unwrap_or_default();

        let query = format!(
            r#"SELECT
                id, agent_type, decision_type, target_service, symbol,
                recommendation, reasoning, confidence, action_data,
                status, priority, created_at, processed_at,
                llm_model, processing_time_ms, cost_usd
            FROM agent_decisions
            WHERE created_at >= NOW() - INTERVAL '{hours} hours'
            {agent_filter}
            ORDER BY created_at DESC
            LIMIT {limit}"#
        );

        let rows = self.model_db
            .query_all(sea_orm::Statement::from_string(backend, query))
            .await
            .map_err(db_err)?;

        let decisions: Vec<DecisionDto> = rows.iter().map(|row| {
            DecisionDto {
                id: opt_i64(row, 0),
                agent_type: opt_str(row, 1).unwrap_or_default(),
                decision_type: opt_str(row, 2),
                target_service: opt_str(row, 3),
                symbol: opt_str(row, 4),
                recommendation: opt_str(row, 5),
                reasoning: opt_str(row, 6),
                confidence: opt_f64(row, 7),
                action_data: opt_json(row, 8),
                status: opt_str(row, 9),
                priority: opt_str(row, 10),
                created_at: opt_ts(row, 11),
                processed_at: opt_ts(row, 12),
                llm_model: opt_str(row, 13),
                processing_time_ms: opt_f64(row, 14),
                cost_usd: opt_f64(row, 15),
            }
        }).collect();

        Ok(RecentDecisionsResponse { decisions })
    }

    // ================================================================
    // GET /agent-analytics/v1/decisions/timeline
    // ================================================================
    pub async fn get_decisions_timeline(
        &self,
        _ctx: &SecurityContext,
        days: i64,
        agent_type: Option<&str>,
    ) -> Result<DecisionsTimelineResponse, DomainError> {
        let backend = self.model_db.get_database_backend();

        let agent_filter = agent_type
            .map(|a| format!(" AND agent_type = '{}'", a.replace('\'', "''")))
            .unwrap_or_default();

        let query = format!(
            r#"SELECT
                DATE_TRUNC('hour', created_at) as hour,
                COUNT(*) as decisions_count,
                AVG(confidence) as avg_confidence,
                SUM(CASE WHEN status = 'pending' THEN 1 ELSE 0 END) as pending,
                SUM(CASE WHEN status = 'applied' THEN 1 ELSE 0 END) as applied,
                SUM(CASE WHEN status = 'rejected' THEN 1 ELSE 0 END) as rejected,
                SUM(CASE WHEN status = 'acknowledged' THEN 1 ELSE 0 END) as acknowledged
            FROM agent_decisions
            WHERE created_at >= CURRENT_DATE - INTERVAL '{days} days'
            {agent_filter}
            GROUP BY DATE_TRUNC('hour', created_at)
            ORDER BY hour DESC"#
        );

        let rows = self.model_db
            .query_all(sea_orm::Statement::from_string(backend, query))
            .await
            .map_err(db_err)?;

        let timeline: Vec<TimelinePointDto> = rows.iter().map(|row| {
            TimelinePointDto {
                hour: opt_ts(row, 0).unwrap_or_default(),
                decisions_count: opt_i64(row, 1),
                avg_confidence: opt_f64(row, 2),
                pending: opt_i64(row, 3),
                applied: opt_i64(row, 4),
                rejected: opt_i64(row, 5),
                acknowledged: opt_i64(row, 6),
            }
        }).collect();

        Ok(DecisionsTimelineResponse { timeline })
    }

    // ================================================================
    // GET /agent-analytics/v1/sentiment/trends
    // ================================================================
    pub async fn get_sentiment_trends(
        &self,
        _ctx: &SecurityContext,
        symbol: Option<&str>,
        hours: i64,
    ) -> Result<SentimentTrendsResponse, DomainError> {
        let backend = self.model_db.get_database_backend();

        let sym_filter = symbol
            .map(|s| format!(" AND symbol = '{}'", s.replace('\'', "''")))
            .unwrap_or_default();

        let detail_query = format!(
            r#"SELECT
                id, symbol, sentiment_score, sentiment_source,
                positive_signals, negative_signals, created_at
            FROM market_sentiment
            WHERE created_at >= NOW() - INTERVAL '{hours} hours'
            {sym_filter}
            ORDER BY created_at DESC
            LIMIT 200"#
        );

        let rows = self.model_db
            .query_all(sea_orm::Statement::from_string(backend, detail_query))
            .await
            .map_err(db_err)?;

        let sentiment_data: Vec<SentimentDataDto> = rows.iter().map(|row| {
            SentimentDataDto {
                id: opt_i64(row, 0),
                symbol: opt_str(row, 1).unwrap_or_default(),
                sentiment_score: opt_f64(row, 2),
                sentiment_source: opt_str(row, 3),
                positive_signals: opt_json(row, 4),
                negative_signals: opt_json(row, 5),
                created_at: opt_ts(row, 6),
            }
        }).collect();

        // Aggregated by hour
        let agg_query = format!(
            r#"SELECT
                DATE_TRUNC('hour', created_at) as hour,
                AVG(sentiment_score) as avg_sentiment,
                COUNT(DISTINCT symbol) as symbols_analyzed,
                COUNT(*) as total_readings
            FROM market_sentiment
            WHERE created_at >= NOW() - INTERVAL '{hours} hours'
            {sym_filter}
            GROUP BY DATE_TRUNC('hour', created_at)
            ORDER BY hour DESC"#
        );

        let agg_rows = self.model_db
            .query_all(sea_orm::Statement::from_string(backend, agg_query))
            .await
            .map_err(db_err)?;

        let aggregated_by_hour: Vec<SentimentAggDto> = agg_rows.iter().map(|row| {
            SentimentAggDto {
                hour: opt_ts(row, 0).unwrap_or_default(),
                avg_sentiment: opt_f64(row, 1),
                symbols_analyzed: opt_i64(row, 2),
                total_readings: opt_i64(row, 3),
            }
        }).collect();

        Ok(SentimentTrendsResponse { sentiment_data, aggregated_by_hour })
    }

    // ================================================================
    // GET /agent-analytics/v1/performance/metrics
    // ================================================================
    pub async fn get_performance_metrics(
        &self,
        _ctx: &SecurityContext,
        days: i64,
        agent_type: Option<&str>,
    ) -> Result<PerformanceMetricsResponse, DomainError> {
        let backend = self.model_db.get_database_backend();

        let agent_filter = agent_type
            .map(|a| format!(" AND agent_type = '{}'", a.replace('\'', "''")))
            .unwrap_or_default();

        let query = format!(
            r#"SELECT
                agent_type, date, decisions_made, decisions_applied,
                decisions_rejected, avg_confidence, avg_processing_time_ms,
                total_cost_usd, positive_outcomes, negative_outcomes,
                neutral_outcomes,
                CASE
                    WHEN decisions_made > 0
                    THEN (decisions_applied::FLOAT / decisions_made) * 100
                    ELSE 0
                END as success_rate
            FROM agent_performance
            WHERE date >= CURRENT_DATE - INTERVAL '{days} days'
            {agent_filter}
            ORDER BY date DESC, agent_type"#
        );

        let rows = self.model_db
            .query_all(sea_orm::Statement::from_string(backend, query))
            .await
            .map_err(db_err)?;

        let performance: Vec<PerformanceMetricDto> = rows.iter().map(|row| {
            PerformanceMetricDto {
                agent_type: opt_str(row, 0).unwrap_or_default(),
                date: opt_date(row, 1),
                decisions_made: opt_i64(row, 2),
                decisions_applied: opt_i64(row, 3),
                decisions_rejected: opt_i64(row, 4),
                avg_confidence: opt_f64(row, 5),
                avg_processing_time_ms: opt_f64(row, 6),
                total_cost_usd: opt_f64(row, 7),
                positive_outcomes: opt_i64(row, 8),
                negative_outcomes: opt_i64(row, 9),
                neutral_outcomes: opt_i64(row, 10),
                success_rate: opt_f64(row, 11).unwrap_or(0.0),
            }
        }).collect();

        Ok(PerformanceMetricsResponse { performance })
    }

    // ================================================================
    // GET /agent-analytics/v1/llm/usage
    // ================================================================
    pub async fn get_llm_usage(
        &self,
        _ctx: &SecurityContext,
        days: i64,
        agent_type: Option<&str>,
    ) -> Result<LlmUsageResponse, DomainError> {
        let backend = self.model_db.get_database_backend();

        let agent_filter = agent_type
            .map(|a| format!(" AND agent_type = '{}'", a.replace('\'', "''")))
            .unwrap_or_default();

        // Usage by day
        let day_query = format!(
            r#"SELECT
                DATE(created_at) as date,
                COUNT(*) as total_requests,
                SUM(input_tokens) as input_tokens,
                SUM(output_tokens) as output_tokens,
                SUM(cost_usd) as total_cost_usd,
                AVG(processing_time_ms) as avg_processing_time_ms
            FROM llm_usage
            WHERE created_at >= CURRENT_DATE - INTERVAL '{days} days'
            {agent_filter}
            GROUP BY DATE(created_at)
            ORDER BY date DESC"#
        );

        let day_rows = self.model_db
            .query_all(sea_orm::Statement::from_string(backend, day_query))
            .await
            .map_err(db_err)?;

        let usage_by_day: Vec<UsageByDayDto> = day_rows.iter().map(|row| {
            UsageByDayDto {
                date: opt_date(row, 0),
                total_requests: opt_i64(row, 1),
                input_tokens: opt_i64(row, 2),
                output_tokens: opt_i64(row, 3),
                total_cost_usd: opt_f64(row, 4),
                avg_processing_time_ms: opt_f64(row, 5),
            }
        }).collect();

        // Usage by agent (no agent_type filter on this one)
        let agent_query = format!(
            r#"SELECT
                agent_type,
                COUNT(*) as total_requests,
                SUM(cost_usd) as total_cost_usd,
                SUM(input_tokens) as total_input_tokens,
                SUM(output_tokens) as total_output_tokens
            FROM llm_usage
            WHERE created_at >= CURRENT_DATE - INTERVAL '{days} days'
            GROUP BY agent_type
            ORDER BY total_cost_usd DESC"#
        );

        let agent_rows = self.model_db
            .query_all(sea_orm::Statement::from_string(backend, agent_query))
            .await
            .map_err(db_err)?;

        let usage_by_agent: Vec<UsageByAgentDto> = agent_rows.iter().map(|row| {
            UsageByAgentDto {
                agent_type: opt_str(row, 0).unwrap_or_default(),
                total_requests: opt_i64(row, 1),
                total_cost_usd: opt_f64(row, 2),
                total_input_tokens: opt_i64(row, 3),
                total_output_tokens: opt_i64(row, 4),
            }
        }).collect();

        // Usage by model
        let model_query = format!(
            r#"SELECT
                llm_model,
                COUNT(*) as total_requests,
                SUM(cost_usd) as total_cost_usd,
                SUM(input_tokens) as total_input_tokens,
                SUM(output_tokens) as total_output_tokens
            FROM llm_usage
            WHERE created_at >= CURRENT_DATE - INTERVAL '{days} days'
            {agent_filter}
            GROUP BY llm_model
            ORDER BY total_cost_usd DESC"#
        );

        let model_rows = self.model_db
            .query_all(sea_orm::Statement::from_string(backend, model_query))
            .await
            .map_err(db_err)?;

        let usage_by_model: Vec<UsageByModelDto> = model_rows.iter().map(|row| {
            UsageByModelDto {
                llm_model: opt_str(row, 0).unwrap_or_default(),
                total_requests: opt_i64(row, 1),
                total_cost_usd: opt_f64(row, 2),
                total_input_tokens: opt_i64(row, 3),
                total_output_tokens: opt_i64(row, 4),
            }
        }).collect();

        Ok(LlmUsageResponse { usage_by_day, usage_by_agent, usage_by_model })
    }

    // ================================================================
    // GET /agent-analytics/v1/config
    // ================================================================
    pub async fn get_agent_config(
        &self,
        _ctx: &SecurityContext,
    ) -> Result<AgentConfigResponse, DomainError> {
        let backend = self.model_db.get_database_backend();

        let rows = self.model_db
            .query_all(sea_orm::Statement::from_string(backend, r#"
                SELECT
                    agent_type, enabled, mode, decision_frequency_minutes,
                    confidence_threshold, config, llm_provider, llm_model, updated_at
                FROM agent_config
                ORDER BY agent_type
            "#.to_string()))
            .await
            .map_err(db_err)?;

        let agents: Vec<AgentConfigDto> = rows.iter().map(|row| {
            AgentConfigDto {
                agent_type: opt_str(row, 0).unwrap_or_default(),
                enabled: opt_bool(row, 1),
                mode: opt_str(row, 2),
                decision_frequency_minutes: opt_i32(row, 3),
                confidence_threshold: opt_f64(row, 4),
                config: opt_json(row, 5),
                llm_provider: opt_str(row, 6),
                llm_model: opt_str(row, 7),
                updated_at: opt_ts(row, 8),
            }
        }).collect();

        Ok(AgentConfigResponse { agents })
    }

    // ================================================================
    // GET /agent-analytics/v1/strategy-research/experiments
    // ================================================================
    pub async fn get_strategy_experiments(
        &self,
        _ctx: &SecurityContext,
        status: Option<&str>,
        limit: i64,
    ) -> Result<StrategyExperimentsResponse, DomainError> {
        let backend = self.model_db.get_database_backend();

        let status_filter = status
            .map(|s| format!(" AND status = '{}'", s.replace('\'', "''")))
            .unwrap_or_default();

        let query = format!(
            r#"SELECT
                id, strategy_name, description, parameters, status,
                created_by, backtest_results, performance_metrics,
                pair, baseline_comparison, improvement_pct,
                promoted_at, retired_at, created_at, updated_at
            FROM strategy_experiments
            WHERE 1=1 {status_filter}
            ORDER BY created_at DESC
            LIMIT {limit}"#
        );

        let rows = self.model_db
            .query_all(sea_orm::Statement::from_string(backend, query))
            .await
            .map_err(db_err)?;

        let experiments: Vec<ExperimentDto> = rows.iter().map(|row| {
            ExperimentDto {
                id: opt_i64(row, 0),
                strategy_name: opt_str(row, 1).unwrap_or_default(),
                description: opt_str(row, 2),
                parameters: opt_json(row, 3),
                status: opt_str(row, 4),
                created_by: opt_str(row, 5),
                backtest_results: opt_json(row, 6),
                performance_metrics: opt_json(row, 7),
                pair: opt_str(row, 8),
                baseline_comparison: opt_json(row, 9),
                improvement_pct: opt_f64(row, 10),
                promoted_at: opt_ts(row, 11),
                retired_at: opt_ts(row, 12),
                created_at: opt_ts(row, 13),
                updated_at: opt_ts(row, 14),
            }
        }).collect();

        // Status counts
        let count_rows = self.model_db
            .query_all(sea_orm::Statement::from_string(backend, r#"
                SELECT status, COUNT(*) as count
                FROM strategy_experiments
                GROUP BY status ORDER BY count DESC
            "#.to_string()))
            .await
            .map_err(db_err)?;

        let status_counts: Vec<StatusCountDto> = count_rows.iter().map(|row| {
            StatusCountDto {
                status: opt_str(row, 0).unwrap_or_default(),
                count: opt_i64(row, 1),
            }
        }).collect();

        Ok(StrategyExperimentsResponse { experiments, status_counts })
    }

    // ================================================================
    // GET /agent-analytics/v1/strategy-research/leaderboard
    // ================================================================
    pub async fn get_strategy_leaderboard(
        &self,
        _ctx: &SecurityContext,
        limit: i64,
    ) -> Result<StrategyLeaderboardResponse, DomainError> {
        let backend = self.model_db.get_database_backend();

        let query = format!(
            r#"SELECT
                id, strategy_name, pair, win_rate, sharpe_ratio,
                total_pnl, total_trades, avg_trade_duration_minutes,
                max_drawdown, profit_factor, avg_return_pct,
                last_evaluated, rank, score, created_at, updated_at
            FROM strategy_leaderboard
            ORDER BY score DESC NULLS LAST, win_rate DESC
            LIMIT {limit}"#
        );

        let rows = self.model_db
            .query_all(sea_orm::Statement::from_string(backend, query))
            .await
            .map_err(db_err)?;

        let leaderboard: Vec<LeaderboardEntryDto> = rows.iter().map(|row| {
            LeaderboardEntryDto {
                id: opt_i64(row, 0),
                strategy_name: opt_str(row, 1).unwrap_or_default(),
                pair: opt_str(row, 2),
                win_rate: opt_f64(row, 3),
                sharpe_ratio: opt_f64(row, 4),
                total_pnl: opt_f64(row, 5),
                total_trades: row.try_get_by_index::<i64>(6).ok(),
                avg_trade_duration_minutes: opt_f64(row, 7),
                max_drawdown: opt_f64(row, 8),
                profit_factor: opt_f64(row, 9),
                avg_return_pct: opt_f64(row, 10),
                last_evaluated: opt_ts(row, 11),
                rank: opt_i32(row, 12),
                score: opt_f64(row, 13),
                created_at: opt_ts(row, 14),
                updated_at: opt_ts(row, 15),
            }
        }).collect();

        Ok(StrategyLeaderboardResponse { leaderboard })
    }

    // ================================================================
    // GET /agent-analytics/v1/strategy-research/indicators
    // ================================================================
    pub async fn get_indicator_analysis(
        &self,
        _ctx: &SecurityContext,
        pair: Option<&str>,
        limit: i64,
    ) -> Result<IndicatorAnalysisResponse, DomainError> {
        let backend = self.model_db.get_database_backend();

        let pair_filter = pair
            .map(|p| format!(" AND pair = '{}'", p.replace('\'', "''")))
            .unwrap_or_default();

        let query = format!(
            r#"SELECT
                id, indicator_name, importance_score, correlation_with_profit,
                optimal_params, combination_group, pair, sample_size,
                evaluation_period_days, notes, evaluated_at, created_at
            FROM indicator_analysis
            WHERE 1=1 {pair_filter}
            ORDER BY importance_score DESC NULLS LAST
            LIMIT {limit}"#
        );

        let rows = self.model_db
            .query_all(sea_orm::Statement::from_string(backend, query))
            .await
            .map_err(db_err)?;

        let indicators: Vec<IndicatorDto> = rows.iter().map(|row| {
            IndicatorDto {
                id: opt_i64(row, 0),
                indicator_name: opt_str(row, 1).unwrap_or_default(),
                importance_score: opt_f64(row, 2),
                correlation_with_profit: opt_f64(row, 3),
                optimal_params: opt_json(row, 4),
                combination_group: opt_str(row, 5),
                pair: opt_str(row, 6),
                sample_size: row.try_get_by_index::<i64>(7).ok(),
                evaluation_period_days: opt_i32(row, 8),
                notes: opt_str(row, 9),
                evaluated_at: opt_ts(row, 10),
                created_at: opt_ts(row, 11),
            }
        }).collect();

        Ok(IndicatorAnalysisResponse { indicators })
    }
}
