use std::collections::HashMap;
use std::sync::Arc;

use sea_orm::{ConnectionTrait, DatabaseConnection, EntityTrait, ColumnTrait, QueryFilter, QueryOrder};
use sea_orm::sea_query::Expr;
use modkit_db::DBProvider;
use modkit_db::DbError;
use modkit_db::secure::{AccessScope, SecureEntityExt, SecureUpdateExt};
use modkit_security::SecurityContext;


use crate::config::ConfigManagerConfig;
use crate::infra::storage::entity::{buy_orders, df_view, pairs_exclusion_list, trade_pairs};
use crate::api::rest::dto::*;
use crate::domain::error::DomainError;

fn db_err(e: impl std::fmt::Display) -> DomainError {
    DomainError::Database(e.to_string())
}

/// Core service containing all config-manager business logic.
pub struct ConfigManagerService {
    db: Arc<DBProvider<DbError>>,
    /// Raw SeaORM connection to Binance_Klines for dynamic table queries.
    klines_conn: Option<DatabaseConnection>,
    cfg: ConfigManagerConfig,
}

impl ConfigManagerService {
    pub fn new(
        db: Arc<DBProvider<DbError>>,
        klines_conn: Option<DatabaseConnection>,
        cfg: ConfigManagerConfig,
    ) -> Self {
        Self { db, klines_conn, cfg }
    }

    // ================================================================
    // Config file helpers
    // ================================================================

    fn config_path(&self, env: &str) -> Result<String, DomainError> {
        match env {
            "production" | "" => Ok(self.cfg.config_path.clone()),
            "testnet" | "testnet_scalping" => {
                self.cfg.config_path_testnet.clone().ok_or_else(|| {
                    DomainError::InvalidConfig("No testnet config path configured".into())
                })
            }
            _ => Err(DomainError::InvalidConfig(format!("Unknown environment: {env}"))),
        }
    }

    fn load_yaml(&self, env: &str) -> Result<serde_yaml::Value, DomainError> {
        let path = self.config_path(env)?;
        let content = std::fs::read_to_string(&path)
            .map_err(|e| DomainError::Io(format!("Failed to read {path}: {e}")))?;
        serde_yaml::from_str(&content)
            .map_err(|e| DomainError::InvalidConfig(format!("Invalid YAML: {e}")))
    }

    fn save_yaml(&self, env: &str, value: &serde_yaml::Value) -> Result<(), DomainError> {
        let path = self.config_path(env)?;
        let content = serde_yaml::to_string(value)
            .map_err(|e| DomainError::InvalidConfig(format!("YAML serialize error: {e}")))?;
        std::fs::write(&path, &content)
            .map_err(|e| DomainError::Io(format!("Failed to write {path}: {e}")))?;
        Ok(())
    }

    fn yaml_bool(v: &serde_yaml::Value, key: &str, default: bool) -> bool {
        match v.get(key) {
            Some(serde_yaml::Value::Bool(b)) => *b,
            Some(serde_yaml::Value::Number(n)) => n.as_i64().unwrap_or(0) != 0,
            _ => default,
        }
    }

    fn yaml_i64(v: &serde_yaml::Value, key: &str, default: i64) -> i64 {
        v.get(key).and_then(|v| v.as_i64()).unwrap_or(default)
    }

    fn yaml_f64(v: &serde_yaml::Value, key: &str, default: f64) -> f64 {
        v.get(key).and_then(|v| v.as_f64()).unwrap_or(default)
    }

    fn yaml_str(v: &serde_yaml::Value, key: &str, default: &str) -> String {
        v.get(key).and_then(|v| v.as_str()).unwrap_or(default).to_string()
    }

    // ================================================================
    // GET /config
    // ================================================================

    pub async fn get_config(
        &self,
        _ctx: &SecurityContext,
        env: &str,
    ) -> Result<ConfigResponse, DomainError> {
        let cfg = self.load_yaml(env)?;
        let env_name = if env.is_empty() { "production" } else { env };

        Ok(ConfigResponse {
            trading_enabled: Self::yaml_bool(&cfg, "trading_enabled", false),
            buying_enabled: Self::yaml_bool(&cfg, "buying_enabled", false),
            sell_signal_enabled: Self::yaml_bool(&cfg, "sell_signal_enabled", false),
            check_price_increase: Self::yaml_bool(&cfg, "check_price_increase", false),
            check_price_drop: Self::yaml_bool(&cfg, "check_price_drop", false),
            periodic_sl_check: Self::yaml_bool(&cfg, "periodic_SL_check", false),
            periodic_sl_check_percent: Self::yaml_f64(&cfg, "periodic_SL_check_percent", 2.0),
            tp_percent_buy: Self::yaml_f64(&cfg, "tp_percent_buy", 1.0),
            consider_tp_percent_buy: Self::yaml_bool(&cfg, "consider_tp_percent_buy", false),
            cut_off_limit: Self::yaml_i64(&cfg, "cut_off_limit", 1000),
            max_buy_orders: Self::yaml_i64(&cfg, "max_buy_orders", 1),
            max_buy_orders_all_coins: Self::yaml_i64(&cfg, "max_buy_orders_all_coins", 10),
            max_buy_orders_price_increase: Self::yaml_i64(&cfg, "max_buy_orders_price_increase", 10),
            max_buy_orders_ml: Self::yaml_i64(&cfg, "max_buy_orders_ml", 10),
            time_between_buys_all_coins: Self::yaml_i64(&cfg, "time_between_buys_all_coins", 1),
            bid_money: Self::yaml_i64(&cfg, "bid_money", 20),
            bid_money_price_increase: Self::yaml_i64(&cfg, "bid_money_price_increase", 20),
            ml_only_predictions: Self::yaml_bool(&cfg, "ml_only_predictions", false),
            use_exclusion_list: Self::yaml_bool(&cfg, "use_exclusion_list", false),
            exclusion_list_add_percentage: Self::yaml_f64(&cfg, "exclusion_list_add_percentage", -15.0),
            exclusion_list_remove_percentage: Self::yaml_f64(&cfg, "exclusion_list_remove_percentage", -10.0),
            execute_trading_interval: Self::yaml_i64(&cfg, "execute_trading_interval", 3600),
            print_trading_interval: Self::yaml_bool(&cfg, "print_trading_interval", false),
            signals_percent_difference_divider: Self::yaml_f64(&cfg, "signals_percent_difference_divider", 5.0),
            signals_percent_difference_hours: Self::yaml_f64(&cfg, "signals_percent_difference_hours", 72.0),
            ema_short: Self::yaml_i64(&cfg, "ema_short", 12),
            ema_med: Self::yaml_i64(&cfg, "ema_med", 72),
            ema_long: Self::yaml_i64(&cfg, "ema_long", 144),
            sma_short: Self::yaml_i64(&cfg, "sma_short", 5),
            sma_med: Self::yaml_i64(&cfg, "sma_med", 8),
            sma_long: Self::yaml_i64(&cfg, "sma_long", 13),
            macd_fast: Self::yaml_i64(&cfg, "macd_fast", 12),
            macd_sign: Self::yaml_i64(&cfg, "macd_sign", 9),
            macd_slow: Self::yaml_i64(&cfg, "macd_slow", 26),
            mfi: Self::yaml_i64(&cfg, "mfi", 14),
            mfi_overbought: Self::yaml_i64(&cfg, "mfi_overbought", 80),
            mfi_oversold: Self::yaml_i64(&cfg, "mfi_oversold", 20),
            stoch: Self::yaml_i64(&cfg, "stoch", 14),
            stoch_overbought: Self::yaml_i64(&cfg, "stoch_overbought", 70),
            stoch_oversold: Self::yaml_i64(&cfg, "stoch_oversold", 30),
            rsi: Self::yaml_i64(&cfg, "rsi", 14),
            rsi_overbought: Self::yaml_i64(&cfg, "rsi_overbought", 70),
            rsi_oversold: Self::yaml_i64(&cfg, "rsi_oversold", 30),
            adx: Self::yaml_i64(&cfg, "adx", 14),
            cci: Self::yaml_i64(&cfg, "cci", 20),
            bbands: Self::yaml_i64(&cfg, "bbands", 20),
            main_loop_interval: Self::yaml_i64(&cfg, "main_loop_interval", 300),
            buy_strategy_interval: Self::yaml_i64(&cfg, "buy_strategy_interval", 1800),
            sell_strategy_interval: Self::yaml_i64(&cfg, "sell_strategy_interval", 900),
            kline_interval: Self::yaml_str(&cfg, "kline_interval", "5m"),
            periodic_heavy_cycle_interval: Self::yaml_i64(&cfg, "periodic_heavy_cycle_interval", 3),
            ml_confidence_threshold: Self::yaml_f64(&cfg, "ml_confidence_threshold", 0.75),
            ml_min_profit_threshold: Self::yaml_f64(&cfg, "ml_min_profit_threshold", 1.0),
            ml_min_timeframe_alignment: Self::yaml_f64(&cfg, "ml_min_timeframe_alignment", 0.6),
            use_multi_timeframe: Self::yaml_bool(&cfg, "use_multi_timeframe", false),
            use_decision_pipeline: Self::yaml_bool(&cfg, "use_decision_pipeline", false),
            trailing_stop_activation_percent: Self::yaml_f64(&cfg, "trailing_stop_activation_percent", 80.0),
            trailing_stop_distance_percent: Self::yaml_f64(&cfg, "trailing_stop_distance_percent", 0.3),
            max_drawdown_from_peak_percent: Self::yaml_f64(&cfg, "max_drawdown_from_peak_percent", 0.5),
            profit_monitor_threshold: Self::yaml_f64(&cfg, "profit_monitor_threshold", 0.5),
            profit_monitor_high_threshold: Self::yaml_f64(&cfg, "profit_monitor_high_threshold", 80.0),
            profit_monitor_interval_high: Self::yaml_i64(&cfg, "profit_monitor_interval_high", 180),
            profit_monitor_interval_moderate: Self::yaml_i64(&cfg, "profit_monitor_interval_moderate", 300),
            peak_protection_threshold: Self::yaml_f64(&cfg, "peak_protection_threshold", 10.0),
            peak_drawdown_trigger: Self::yaml_f64(&cfg, "peak_drawdown_trigger", 50.0),
            public_ip_address: Self::yaml_str(&cfg, "public_IP_address", ""),
            indicator_cache_ttl_seconds: Self::yaml_i64(&cfg, "indicator_cache_ttl_seconds", 120),
            indicator_cache_max_size: Self::yaml_i64(&cfg, "indicator_cache_max_size", 400),
            environment: env_name.to_string(),
        })
    }

    // ================================================================
    // PUT /config
    // ================================================================

    pub async fn update_config(
        &self,
        _ctx: &SecurityContext,
        env: &str,
        section: &str,
        updates: &HashMap<String, serde_json::Value>,
    ) -> Result<ConfigUpdateResponse, DomainError> {
        let mut cfg = self.load_yaml(env)?;

        let allowed_keys: &[&str] = match section {
            "trade" => &[
                "trading_enabled", "buying_enabled", "sell_signal_enabled",
                "check_price_increase", "check_price_drop", "periodic_SL_check",
                "periodic_SL_check_percent", "tp_percent_buy", "consider_tp_percent_buy",
                "cut_off_limit", "max_buy_orders", "max_buy_orders_all_coins",
                "max_buy_orders_price_increase", "max_buy_orders_ml",
                "time_between_buys_all_coins", "bid_money", "bid_money_price_increase",
                "ml_only_predictions", "use_exclusion_list", "exclusion_list_add_percentage",
                "exclusion_list_remove_percentage", "execute_trading_interval",
                "print_trading_interval", "signals_percent_difference_divider",
                "signals_percent_difference_hours",
            ],
            "indicators" => &[
                "ema_short", "ema_med", "ema_long", "sma_short", "sma_med", "sma_long",
                "macd_fast", "macd_sign", "macd_slow", "mfi", "mfi_overbought", "mfi_oversold",
                "stoch", "stoch_overbought", "stoch_oversold", "rsi", "rsi_overbought",
                "rsi_oversold", "adx", "cci", "bbands",
            ],
            "intervals" => &[
                "main_loop_interval", "buy_strategy_interval", "sell_strategy_interval",
                "kline_interval", "periodic_heavy_cycle_interval",
            ],
            "ml" => &[
                "ml_confidence_threshold", "ml_min_profit_threshold",
                "ml_min_timeframe_alignment", "use_multi_timeframe", "use_decision_pipeline",
            ],
            "profit" => &[
                "trailing_stop_activation_percent", "trailing_stop_distance_percent",
                "max_drawdown_from_peak_percent", "profit_monitor_threshold",
                "profit_monitor_high_threshold", "profit_monitor_interval_high",
                "profit_monitor_interval_moderate", "peak_protection_threshold",
                "peak_drawdown_trigger",
            ],
            "system" => &[
                "public_IP_address", "indicator_cache_ttl_seconds", "indicator_cache_max_size",
            ],
            _ => return Err(DomainError::InvalidConfig(format!("Invalid section: {section}"))),
        };

        // Boolean keys that should be stored as 1/0 in YAML
        let bool_keys: &[&str] = &[
            "trading_enabled", "buying_enabled", "sell_signal_enabled",
            "check_price_increase", "check_price_drop", "periodic_SL_check",
            "consider_tp_percent_buy", "ml_only_predictions", "use_exclusion_list",
            "print_trading_interval", "use_multi_timeframe", "use_decision_pipeline",
        ];

        let mapping = cfg.as_mapping_mut().ok_or_else(|| {
            DomainError::InvalidConfig("Config is not a YAML mapping".into())
        })?;

        let mut updated_keys = Vec::new();
        for key in allowed_keys {
            if let Some(val) = updates.get(*key) {
                let yaml_val = if bool_keys.contains(key) {
                    // Convert bool to 1/0 integer for YAML compat
                    let b = val.as_bool().unwrap_or(false);
                    serde_yaml::Value::Number(serde_yaml::Number::from(if b { 1 } else { 0 }))
                } else {
                    json_to_yaml(val)
                };
                mapping.insert(
                    serde_yaml::Value::String(key.to_string()),
                    yaml_val,
                );
                updated_keys.push(key.to_string());
            }
        }

        self.save_yaml(env, &cfg)?;

        Ok(ConfigUpdateResponse {
            success: true,
            message: format!("Updated {} settings", updated_keys.len()),
            updated_keys,
        })
    }

    // ================================================================
    // GET /config/raw
    // ================================================================

    pub async fn get_config_raw(
        &self,
        _ctx: &SecurityContext,
        env: &str,
    ) -> Result<RawConfigResponse, DomainError> {
        let path = self.config_path(env)?;
        let yaml = std::fs::read_to_string(&path)
            .map_err(|e| DomainError::Io(format!("Failed to read {path}: {e}")))?;
        let env_name = if env.is_empty() { "production" } else { env };
        Ok(RawConfigResponse {
            yaml,
            environment: env_name.to_string(),
        })
    }

    // ================================================================
    // PUT /config/raw
    // ================================================================

    pub async fn update_config_raw(
        &self,
        _ctx: &SecurityContext,
        env: &str,
        yaml_content: &str,
    ) -> Result<SuccessResponse, DomainError> {
        // Validate YAML syntax
        let _: serde_yaml::Value = serde_yaml::from_str(yaml_content)
            .map_err(|e| DomainError::InvalidConfig(format!("Invalid YAML syntax: {e}")))?;

        let path = self.config_path(env)?;
        std::fs::write(&path, yaml_content)
            .map_err(|e| DomainError::Io(format!("Failed to write {path}: {e}")))?;

        Ok(SuccessResponse {
            success: true,
            message: "Configuration updated successfully".to_string(),
        })
    }

    // ================================================================
    // GET /pairs (trade pairs with evolution data)
    // ================================================================

    pub async fn get_trade_pairs(
        &self,
        _ctx: &SecurityContext,
    ) -> Result<TradePairsListDto, DomainError> {
        let scope = AccessScope::default();
        let conn = self.db.conn().map_err(db_err)?;

        // Fetch trade_pairs
        let pairs = trade_pairs::Entity::find()
            .order_by_desc(trade_pairs::Column::LastPurchaseTime)
            .secure()
            .scope_with(&scope)
            .all(&conn)
            .await
            .map_err(db_err)?;

        // Fetch df_view for price evolution
        let df_rows = df_view::Entity::find()
            .secure()
            .scope_with(&scope)
            .all(&conn)
            .await
            .map_err(db_err)?;
        let df_map: HashMap<String, df_view::Model> = df_rows
            .into_iter()
            .map(|r| (r.pair.clone(), r))
            .collect();

        // Open orders per pair (FILLED + no sell)
        let open_buys = buy_orders::Entity::find()
            .filter(buy_orders::Column::Status.eq("FILLED"))
            .filter(buy_orders::Column::HasSellOrder.eq("No"))
            .secure()
            .scope_with(&scope)
            .all(&conn)
            .await
            .map_err(db_err)?;
        let mut open_count: HashMap<String, i64> = HashMap::new();
        for b in &open_buys {
            if let Some(sym) = &b.symbol {
                *open_count.entry(sym.clone()).or_default() += 1;
            }
        }

        // Excluded pairs
        let excluded = pairs_exclusion_list::Entity::find()
            .secure()
            .scope_with(&scope)
            .all(&conn)
            .await
            .map_err(db_err)?;
        let excluded_set: std::collections::HashSet<String> = excluded
            .iter()
            .filter_map(|e| e.pair.clone())
            .collect();

        let result: Vec<TradePairDto> = pairs
            .iter()
            .filter_map(|tp| {
                let pair = tp.pair.as_deref()?;
                let df = df_map.get(pair);
                Some(TradePairDto {
                    pair: pair.to_string(),
                    change_24h: df.and_then(|d| d.one_day_change_percentage).unwrap_or(0.0),
                    excluded: excluded_set.contains(pair),
                    highest_percent: df.and_then(|d| d.highest_percentage).unwrap_or(0.0),
                    highest_price: df.and_then(|d| d.highest_price).unwrap_or(0.0),
                    current_price: df.and_then(|d| d.current_price).unwrap_or(0.0),
                    lowest_price: df.and_then(|d| d.lowes_price).unwrap_or(0.0),
                    lowest_percent: df.and_then(|d| d.lowest_percentage).unwrap_or(0.0),
                    open_orders: *open_count.get(pair).unwrap_or(&0),
                    last_purchase_time: tp.last_purchase_time.map(|t| t.to_string()),
                    last_sell_time: tp.last_sell_time.map(|t| t.to_string()),
                    est_profit_percent: tp.signals_percent_difference.unwrap_or(1.0),
                })
            })
            .collect();

        Ok(TradePairsListDto { pairs: result })
    }

    // ================================================================
    // GET /pairs/{symbol}/signals
    // ================================================================

    pub async fn get_signals(
        &self,
        _ctx: &SecurityContext,
        symbol: &str,
        _percent: f64,
    ) -> Result<SignalsResponse, DomainError> {
        let klines = self.klines_conn.as_ref().ok_or_else(|| {
            DomainError::InvalidConfig("Klines database not configured".into())
        })?;

        // Format pair name for klines table
        let formatted = format_pair_name(symbol);
        let table_name = format!("{formatted}_kline_5m");

        // Use raw query since table names are dynamic
        let backend = sea_orm::DatabaseBackend::Postgres;

        // Check if table exists
        let check_sql = format!(
            "SELECT COUNT(*) as cnt FROM information_schema.tables WHERE table_name = '{table_name}'"
        );
        let check_result = klines
            .query_one(sea_orm::Statement::from_string(backend, check_sql))
            .await
            .map_err(db_err)?;

        let table_exists = check_result
            .and_then(|r| r.try_get_by_index::<i64>(0).ok())
            .unwrap_or(0)
            > 0;

        if !table_exists {
            return Ok(SignalsResponse {
                signals: vec![],
                max_hours: 24,
            });
        }

        // Get last 30 days of 5m kline data
        let data_sql = format!(
            r#"SELECT close_time, close_price, high_price, low_price, open_price, base_asset_volume
               FROM "{table_name}"
               WHERE kline_closed = TRUE
               ORDER BY close_time DESC
               LIMIT 8640"#
        );
        let rows = klines
            .query_all(sea_orm::Statement::from_string(backend, data_sql))
            .await
            .map_err(db_err)?;

        if rows.is_empty() {
            return Ok(SignalsResponse {
                signals: vec![],
                max_hours: 24,
            });
        }

        // Parse rows into OHLCV data (reversed for chronological order)
        let mut ohlcv: Vec<(i64, f64, f64, f64, f64, f64)> = Vec::with_capacity(rows.len());
        for row in rows.iter().rev() {
            let close_time: i64 = row.try_get_by_index::<rust_decimal::Decimal>(0)
                .ok()
                .and_then(|d| d.to_string().parse::<i64>().ok())
                .unwrap_or(0);
            let close: f64 = row.try_get_by_index::<f64>(1).unwrap_or(0.0);
            let high: f64 = row.try_get_by_index::<f64>(2).unwrap_or(0.0);
            let low: f64 = row.try_get_by_index::<f64>(3).unwrap_or(0.0);
            let open: f64 = row.try_get_by_index::<f64>(4).unwrap_or(0.0);
            let vol: f64 = row.try_get_by_index::<f64>(5).unwrap_or(0.0);
            ohlcv.push((close_time, close, high, low, open, vol));
        }

        let max_hours = if ohlcv.len() > 1 {
            let first = ohlcv.first().unwrap().0;
            let last = ohlcv.last().unwrap().0;
            let diff = (last - first).max(1);
            // close_time could be milliseconds or seconds
            let hours = if diff > 1_000_000_000_000 {
                diff / (1000 * 60 * 60)
            } else {
                diff / 3600
            };
            hours.min(720).max(1)
        } else {
            24
        };

        // Calculate technical indicators
        let closes: Vec<f64> = ohlcv.iter().map(|r| r.1).collect();
        let highs: Vec<f64> = ohlcv.iter().map(|r| r.2).collect();
        let lows: Vec<f64> = ohlcv.iter().map(|r| r.3).collect();
        let volumes: Vec<f64> = ohlcv.iter().map(|r| r.5).collect();

        let rsi_vals = calc_rsi(&closes, 14);
        let ema_short = calc_ema(&closes, 12);
        let ema_med = calc_ema(&closes, 72);
        let ema_long = calc_ema(&closes, 144);
        let sma_short = calc_sma(&closes, 5);
        let sma_med = calc_sma(&closes, 8);
        let sma_long = calc_sma(&closes, 13);
        let (macd_line, macd_sig, macd_diff) = calc_macd(&closes, 12, 26, 9);
        let (bb_high, bb_low, bb_med) = calc_bbands(&closes, 20);
        let (stoch_k, stoch_d) = calc_stochastic(&highs, &lows, &closes, 14);
        let cci_vals = calc_cci(&highs, &lows, &closes, 20);
        let mfi_vals = calc_mfi(&highs, &lows, &closes, &volumes, 14);
        let adx_vals = calc_adx(&highs, &lows, &closes, 14);
        let obv_vals = calc_obv(&closes, &volumes);
        let vol_mean = calc_sma(&volumes, 3);

        let signals: Vec<SignalPointDto> = ohlcv
            .iter()
            .enumerate()
            .map(|(i, (ct, close, high, low, open, vol))| {
                // Convert ms timestamp to ISO string
                let ts_secs = if *ct > 1_000_000_000_000 { ct / 1000 } else { *ct };
                let dt = chrono::DateTime::from_timestamp(ts_secs, 0)
                    .map(|d| d.format("%Y-%m-%dT%H:%M:%SZ").to_string())
                    .unwrap_or_else(|| ct.to_string());

                SignalPointDto {
                    close_time: dt,
                    close_price: *close,
                    high_price: *high,
                    low_price: *low,
                    open_price: *open,
                    base_asset_volume: *vol,
                    rsi: rsi_vals.get(i).and_then(|v| *v),
                    ema_short: ema_short.get(i).and_then(|v| *v),
                    ema_med: ema_med.get(i).and_then(|v| *v),
                    ema_long: ema_long.get(i).and_then(|v| *v),
                    sma_short: sma_short.get(i).and_then(|v| *v),
                    sma_med: sma_med.get(i).and_then(|v| *v),
                    sma_long: sma_long.get(i).and_then(|v| *v),
                    macd: macd_line.get(i).and_then(|v| *v),
                    macd_signal: macd_sig.get(i).and_then(|v| *v),
                    macd_diff: macd_diff.get(i).and_then(|v| *v),
                    bbands_high: bb_high.get(i).and_then(|v| *v),
                    bbands_low: bb_low.get(i).and_then(|v| *v),
                    bbands_med: bb_med.get(i).and_then(|v| *v),
                    stoch: stoch_k.get(i).and_then(|v| *v),
                    stoch_ma: stoch_d.get(i).and_then(|v| *v),
                    cci: cci_vals.get(i).and_then(|v| *v),
                    mfi: mfi_vals.get(i).and_then(|v| *v),
                    adx: adx_vals.get(i).and_then(|v| *v),
                    obv: obv_vals.get(i).and_then(|v| *v),
                    volume_mean: vol_mean.get(i).and_then(|v| *v),
                }
            })
            .collect();

        Ok(SignalsResponse {
            signals,
            max_hours,
        })
    }

    // ================================================================
    // PUT /pairs/{symbol}/profit-percent
    // ================================================================

    pub async fn update_profit_percent(
        &self,
        _ctx: &SecurityContext,
        symbol: &str,
        percent: f64,
    ) -> Result<ProfitPercentUpdateResponse, DomainError> {
        let scope = AccessScope::default();
        let conn = self.db.conn().map_err(db_err)?;

        // Verify the trade pair exists
        let _pair = trade_pairs::Entity::find()
            .filter(trade_pairs::Column::Pair.eq(symbol))
            .secure()
            .scope_with(&scope)
            .one(&conn)
            .await
            .map_err(db_err)?
            .ok_or_else(|| DomainError::NotFound(format!("Pair {symbol} not found")))?;

        // Update via secure update_many
        trade_pairs::Entity::update_many()
            .col_expr(trade_pairs::Column::SignalsPercentDifference, Expr::value(Some(percent)))
            .filter(trade_pairs::Column::Pair.eq(symbol))
            .secure()
            .scope_with(&scope)
            .exec(&conn)
            .await
            .map_err(db_err)?;

        Ok(ProfitPercentUpdateResponse {
            success: true,
            message: format!("Updated profit percent for {symbol} to {percent}%"),
        })
    }

    // ================================================================
    // POST /restart
    // ================================================================

    pub async fn restart_container(
        &self,
        _ctx: &SecurityContext,
        env: &str,
    ) -> Result<RestartResponse, DomainError> {
        let container = match env {
            "production" | "" => "cricoai-prod",
            "testnet" => "cricoai-testnet",
            "testnet_scalping" => "cricoai-testnet",
            _ => return Err(DomainError::InvalidConfig(format!("Unknown environment: {env}"))),
        };

        let output = std::process::Command::new("docker")
            .args(["restart", container])
            .output()
            .map_err(|e| DomainError::Io(format!("Failed to run docker restart: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(DomainError::Io(format!("Docker restart failed: {stderr}")));
        }

        let env_name = if env.is_empty() { "production" } else { env };
        Ok(RestartResponse {
            success: true,
            message: format!("Container {container} restarted successfully"),
            container: container.to_string(),
            environment: env_name.to_string(),
        })
    }
}

// ================================================================
// Helper: JSON value -> YAML value
// ================================================================

fn json_to_yaml(v: &serde_json::Value) -> serde_yaml::Value {
    match v {
        serde_json::Value::Null => serde_yaml::Value::Null,
        serde_json::Value::Bool(b) => serde_yaml::Value::Bool(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                serde_yaml::Value::Number(serde_yaml::Number::from(i))
            } else if let Some(f) = n.as_f64() {
                serde_yaml::Value::Number(serde_yaml::Number::from(f))
            } else {
                serde_yaml::Value::Null
            }
        }
        serde_json::Value::String(s) => serde_yaml::Value::String(s.clone()),
        _ => serde_yaml::Value::String(v.to_string()),
    }
}

// ================================================================
// Helper: format pair name for kline table lookup
// ================================================================

fn format_pair_name(pair: &str) -> String {
    let formatted = pair.to_lowercase();
    if formatted.starts_with("1000") {
        format!("n{}", &formatted[4..])
    } else if formatted.starts_with('1') {
        format!("one{}", &formatted[1..])
    } else {
        formatted
    }
}

// ================================================================
// Technical indicator calculations
// ================================================================

fn calc_sma(data: &[f64], period: usize) -> Vec<Option<f64>> {
    let mut result = vec![None; data.len()];
    for i in (period - 1)..data.len() {
        let sum: f64 = data[(i + 1 - period)..=i].iter().sum();
        result[i] = Some(sum / period as f64);
    }
    result
}

fn calc_ema(data: &[f64], period: usize) -> Vec<Option<f64>> {
    if data.is_empty() || period == 0 {
        return vec![None; data.len()];
    }
    let mut result = vec![None; data.len()];
    let k = 2.0 / (period as f64 + 1.0);
    // Seed with SMA
    if data.len() >= period {
        let sma: f64 = data[..period].iter().sum::<f64>() / period as f64;
        result[period - 1] = Some(sma);
        let mut prev = sma;
        for i in period..data.len() {
            let ema = data[i] * k + prev * (1.0 - k);
            result[i] = Some(ema);
            prev = ema;
        }
    }
    result
}

fn calc_rsi(data: &[f64], period: usize) -> Vec<Option<f64>> {
    let mut result = vec![None; data.len()];
    if data.len() < period + 1 {
        return result;
    }
    let mut gains = Vec::with_capacity(data.len());
    let mut losses = Vec::with_capacity(data.len());
    gains.push(0.0);
    losses.push(0.0);
    for i in 1..data.len() {
        let diff = data[i] - data[i - 1];
        gains.push(if diff > 0.0 { diff } else { 0.0 });
        losses.push(if diff < 0.0 { -diff } else { 0.0 });
    }
    let mut avg_gain: f64 = gains[1..=period].iter().sum::<f64>() / period as f64;
    let mut avg_loss: f64 = losses[1..=period].iter().sum::<f64>() / period as f64;
    if avg_loss == 0.0 {
        result[period] = Some(100.0);
    } else {
        result[period] = Some(100.0 - 100.0 / (1.0 + avg_gain / avg_loss));
    }
    for i in (period + 1)..data.len() {
        avg_gain = (avg_gain * (period as f64 - 1.0) + gains[i]) / period as f64;
        avg_loss = (avg_loss * (period as f64 - 1.0) + losses[i]) / period as f64;
        if avg_loss == 0.0 {
            result[i] = Some(100.0);
        } else {
            result[i] = Some(100.0 - 100.0 / (1.0 + avg_gain / avg_loss));
        }
    }
    result
}

fn calc_macd(data: &[f64], fast: usize, slow: usize, signal: usize) -> (Vec<Option<f64>>, Vec<Option<f64>>, Vec<Option<f64>>) {
    let ema_fast = calc_ema(data, fast);
    let ema_slow = calc_ema(data, slow);
    let mut macd_line: Vec<Option<f64>> = vec![None; data.len()];
    for i in 0..data.len() {
        if let (Some(f), Some(s)) = (ema_fast[i], ema_slow[i]) {
            macd_line[i] = Some(f - s);
        }
    }
    let macd_vals: Vec<f64> = macd_line.iter().filter_map(|v| *v).collect();
    let sig = calc_ema(&macd_vals, signal);
    let mut macd_sig = vec![None; data.len()];
    let mut macd_diff = vec![None; data.len()];
    let mut sig_idx = 0;
    for i in 0..data.len() {
        if macd_line[i].is_some() {
            macd_sig[i] = sig.get(sig_idx).copied().flatten();
            if let (Some(m), Some(s)) = (macd_line[i], macd_sig[i]) {
                macd_diff[i] = Some(m - s);
            }
            sig_idx += 1;
        }
    }
    (macd_line, macd_sig, macd_diff)
}

fn calc_bbands(data: &[f64], period: usize) -> (Vec<Option<f64>>, Vec<Option<f64>>, Vec<Option<f64>>) {
    let sma = calc_sma(data, period);
    let mut high = vec![None; data.len()];
    let mut low = vec![None; data.len()];
    let mut mid = vec![None; data.len()];
    for i in (period - 1)..data.len() {
        if let Some(m) = sma[i] {
            let slice = &data[(i + 1 - period)..=i];
            let variance: f64 = slice.iter().map(|x| (x - m).powi(2)).sum::<f64>() / period as f64;
            let std = variance.sqrt();
            high[i] = Some(m + 2.0 * std);
            low[i] = Some(m - 2.0 * std);
            mid[i] = Some(m);
        }
    }
    (high, low, mid)
}

fn calc_stochastic(highs: &[f64], lows: &[f64], closes: &[f64], period: usize) -> (Vec<Option<f64>>, Vec<Option<f64>>) {
    let n = closes.len();
    let mut k = vec![None; n];
    for i in (period - 1)..n {
        let h_max = highs[(i + 1 - period)..=i].iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let l_min = lows[(i + 1 - period)..=i].iter().cloned().fold(f64::INFINITY, f64::min);
        let denom = h_max - l_min;
        k[i] = Some(if denom == 0.0 { 50.0 } else { 100.0 * (closes[i] - l_min) / denom });
    }
    let k_vals: Vec<f64> = k.iter().map(|v| v.unwrap_or(0.0)).collect();
    let d = calc_sma(&k_vals, 3);
    let d: Vec<Option<f64>> = d.into_iter().enumerate().map(|(i, v)| if k[i].is_some() { v } else { None }).collect();
    (k, d)
}

fn calc_cci(highs: &[f64], lows: &[f64], closes: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = closes.len();
    let tp: Vec<f64> = (0..n).map(|i| (highs[i] + lows[i] + closes[i]) / 3.0).collect();
    let mut result = vec![None; n];
    for i in (period - 1)..n {
        let slice = &tp[(i + 1 - period)..=i];
        let mean: f64 = slice.iter().sum::<f64>() / period as f64;
        let mad: f64 = slice.iter().map(|x| (x - mean).abs()).sum::<f64>() / period as f64;
        result[i] = Some(if mad == 0.0 { 0.0 } else { (tp[i] - mean) / (0.015 * mad) });
    }
    result
}

fn calc_mfi(highs: &[f64], lows: &[f64], closes: &[f64], volumes: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = closes.len();
    let tp: Vec<f64> = (0..n).map(|i| (highs[i] + lows[i] + closes[i]) / 3.0).collect();
    let mf: Vec<f64> = tp.iter().zip(volumes).map(|(t, v)| t * v).collect();
    let mut result = vec![None; n];
    for i in period..n {
        let mut pos = 0.0;
        let mut neg = 0.0;
        for j in (i + 1 - period)..=i {
            if j > 0 && tp[j] > tp[j - 1] {
                pos += mf[j];
            } else if j > 0 {
                neg += mf[j];
            }
        }
        result[i] = Some(if neg == 0.0 { 100.0 } else { 100.0 - 100.0 / (1.0 + pos / neg) });
    }
    result
}

fn calc_adx(highs: &[f64], lows: &[f64], closes: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = closes.len();
    if n < period + 1 {
        return vec![None; n];
    }
    // True range
    let mut tr = vec![0.0; n];
    for i in 1..n {
        tr[i] = (highs[i] - lows[i])
            .max((highs[i] - closes[i - 1]).abs())
            .max((lows[i] - closes[i - 1]).abs());
    }
    let atr_vals = calc_sma(&tr, period);

    let mut plus_dm = vec![0.0; n];
    let mut minus_dm = vec![0.0; n];
    for i in 1..n {
        let up = highs[i] - highs[i - 1];
        let down = lows[i - 1] - lows[i];
        plus_dm[i] = if up > down && up > 0.0 { up } else { 0.0 };
        minus_dm[i] = if down > up && down > 0.0 { down } else { 0.0 };
    }
    let plus_di_raw = calc_sma(&plus_dm, period);
    let minus_di_raw = calc_sma(&minus_dm, period);

    let mut dx = vec![0.0; n];
    for i in 0..n {
        if let (Some(atr), Some(pd), Some(md)) = (atr_vals[i], plus_di_raw[i], minus_di_raw[i]) {
            if atr > 0.0 {
                let pdi = 100.0 * pd / atr;
                let mdi = 100.0 * md / atr;
                let sum = pdi + mdi;
                dx[i] = if sum == 0.0 { 0.0 } else { 100.0 * (pdi - mdi).abs() / sum };
            }
        }
    }
    calc_sma(&dx, period)
}

fn calc_obv(closes: &[f64], volumes: &[f64]) -> Vec<Option<f64>> {
    let mut result = vec![None; closes.len()];
    if closes.is_empty() {
        return result;
    }
    let mut obv = 0.0;
    result[0] = Some(0.0);
    for i in 1..closes.len() {
        if closes[i] > closes[i - 1] {
            obv += volumes[i];
        } else if closes[i] < closes[i - 1] {
            obv -= volumes[i];
        }
        result[i] = Some(obv);
    }
    result
}
