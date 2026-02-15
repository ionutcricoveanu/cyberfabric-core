use sea_orm::entity::prelude::*;
use modkit_db_macros::Scopable;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Scopable)]
#[sea_orm(table_name = "prediction_history_v2")]
#[secure(unrestricted)]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub symbol: Option<String>,
    pub prediction_timestamp: Option<DateTime>,
    pub model_version: Option<String>,
    pub entry_price: Option<Decimal>,
    pub market_regime: Option<String>,
    pub market_volatility: Option<Decimal>,
    pub timeframe_primary: Option<String>,
    pub up_probability: Option<Decimal>,
    pub down_probability: Option<Decimal>,
    pub confidence_score: Option<Decimal>,
    pub timeframe_alignment: Option<Decimal>,
    pub predicted_profit_percent: Option<Decimal>,
    pub predicted_stop_loss_percent: Option<Decimal>,
    pub target_price: Option<Decimal>,
    pub stop_loss_price: Option<Decimal>,
    pub features_count: Option<i32>,
    pub feature_importance_top5: Option<String>,
    pub multi_timeframe_features_used: Option<i32>,
    pub resulted_in_trade: Option<bool>,
    pub trade_entry_time: Option<DateTime>,
    pub trade_entry_price: Option<Decimal>,
    pub buy_strategy_used: Option<i32>,
    pub verification_timestamp: Option<DateTime>,
    pub verification_period_hours: Option<i32>,
    pub max_price_reached: Option<Decimal>,
    pub min_price_reached: Option<Decimal>,
    pub price_at_verification: Option<Decimal>,
    pub actual_profit_percent: Option<Decimal>,
    pub max_profit_percent: Option<Decimal>,
    pub max_drawdown_percent: Option<Decimal>,
    pub direction_correct: Option<bool>,
    pub profit_target_hit: Option<bool>,
    pub stop_loss_triggered: Option<bool>,
    pub prediction_quality_score: Option<Decimal>,
    pub feature_drift_score: Option<Decimal>,
    pub model_age_days: Option<i32>,
    pub requires_retraining: Option<bool>,
    pub volume_at_prediction: Option<Decimal>,
    pub rsi_at_prediction: Option<Decimal>,
    pub macd_at_prediction: Option<Decimal>,
    pub created_at: Option<DateTime>,
    pub updated_at: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
