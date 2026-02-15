use std::collections::HashMap;

// ================================================================
// Config endpoints
// ================================================================

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct ConfigResponse {
    pub trading_enabled: bool,
    pub buying_enabled: bool,
    pub sell_signal_enabled: bool,
    pub check_price_increase: bool,
    pub check_price_drop: bool,
    pub periodic_sl_check: bool,
    pub periodic_sl_check_percent: f64,
    pub tp_percent_buy: f64,
    pub consider_tp_percent_buy: bool,
    pub cut_off_limit: i64,
    pub max_buy_orders: i64,
    pub max_buy_orders_all_coins: i64,
    pub max_buy_orders_price_increase: i64,
    pub max_buy_orders_ml: i64,
    pub time_between_buys_all_coins: i64,
    pub bid_money: i64,
    pub bid_money_price_increase: i64,
    pub ml_only_predictions: bool,
    pub use_exclusion_list: bool,
    pub exclusion_list_add_percentage: f64,
    pub exclusion_list_remove_percentage: f64,
    pub execute_trading_interval: i64,
    pub print_trading_interval: bool,
    pub signals_percent_difference_divider: f64,
    pub signals_percent_difference_hours: f64,
    pub ema_short: i64,
    pub ema_med: i64,
    pub ema_long: i64,
    pub sma_short: i64,
    pub sma_med: i64,
    pub sma_long: i64,
    pub macd_fast: i64,
    pub macd_sign: i64,
    pub macd_slow: i64,
    pub mfi: i64,
    pub mfi_overbought: i64,
    pub mfi_oversold: i64,
    pub stoch: i64,
    pub stoch_overbought: i64,
    pub stoch_oversold: i64,
    pub rsi: i64,
    pub rsi_overbought: i64,
    pub rsi_oversold: i64,
    pub adx: i64,
    pub cci: i64,
    pub bbands: i64,
    pub main_loop_interval: i64,
    pub buy_strategy_interval: i64,
    pub sell_strategy_interval: i64,
    pub kline_interval: String,
    pub periodic_heavy_cycle_interval: i64,
    pub ml_confidence_threshold: f64,
    pub ml_min_profit_threshold: f64,
    pub ml_min_timeframe_alignment: f64,
    pub use_multi_timeframe: bool,
    pub use_decision_pipeline: bool,
    pub trailing_stop_activation_percent: f64,
    pub trailing_stop_distance_percent: f64,
    pub max_drawdown_from_peak_percent: f64,
    pub profit_monitor_threshold: f64,
    pub profit_monitor_high_threshold: f64,
    pub profit_monitor_interval_high: i64,
    pub profit_monitor_interval_moderate: i64,
    pub peak_protection_threshold: f64,
    pub peak_drawdown_trigger: f64,
    pub public_ip_address: String,
    pub indicator_cache_ttl_seconds: i64,
    pub indicator_cache_max_size: i64,
    pub environment: String,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(request)]
pub struct ConfigUpdateRequest {
    pub section: String,
    pub config: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct ConfigUpdateResponse {
    pub success: bool,
    pub message: String,
    pub updated_keys: Vec<String>,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct RawConfigResponse {
    pub yaml: String,
    pub environment: String,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(request)]
pub struct RawConfigUpdateRequest {
    pub yaml: String,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct SuccessResponse {
    pub success: bool,
    pub message: String,
}

// ================================================================
// Trade pairs endpoints
// ================================================================

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct TradePairDto {
    pub pair: String,
    pub change_24h: f64,
    pub excluded: bool,
    pub highest_percent: f64,
    pub highest_price: f64,
    pub current_price: f64,
    pub lowest_price: f64,
    pub lowest_percent: f64,
    pub open_orders: i64,
    pub last_purchase_time: Option<String>,
    pub last_sell_time: Option<String>,
    pub est_profit_percent: f64,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct TradePairsListDto {
    pub pairs: Vec<TradePairDto>,
}

// ================================================================
// Signals endpoints
// ================================================================

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct SignalPointDto {
    pub close_time: String,
    pub close_price: f64,
    pub high_price: f64,
    pub low_price: f64,
    pub open_price: f64,
    pub base_asset_volume: f64,
    pub rsi: Option<f64>,
    pub ema_short: Option<f64>,
    pub ema_med: Option<f64>,
    pub ema_long: Option<f64>,
    pub sma_short: Option<f64>,
    pub sma_med: Option<f64>,
    pub sma_long: Option<f64>,
    pub macd: Option<f64>,
    pub macd_signal: Option<f64>,
    pub macd_diff: Option<f64>,
    pub bbands_high: Option<f64>,
    pub bbands_low: Option<f64>,
    pub bbands_med: Option<f64>,
    pub stoch: Option<f64>,
    pub stoch_ma: Option<f64>,
    pub cci: Option<f64>,
    pub mfi: Option<f64>,
    pub adx: Option<f64>,
    pub obv: Option<f64>,
    pub volume_mean: Option<f64>,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct SignalsResponse {
    pub signals: Vec<SignalPointDto>,
    pub max_hours: i64,
}

// ================================================================
// Profit percent update
// ================================================================

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(request)]
pub struct ProfitPercentUpdateRequest {
    pub percent: f64,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct ProfitPercentUpdateResponse {
    pub success: bool,
    pub message: String,
}

// ================================================================
// Restart
// ================================================================

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct RestartResponse {
    pub success: bool,
    pub message: String,
    pub container: String,
    pub environment: String,
}
