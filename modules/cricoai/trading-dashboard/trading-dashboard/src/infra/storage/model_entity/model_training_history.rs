use sea_orm::entity::prelude::*;
use modkit_db_macros::Scopable;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Scopable)]
#[sea_orm(table_name = "model_training_history")]
#[secure(unrestricted)]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub symbol: String,
    pub trained_at: DateTime,
    pub promoted: bool,
    pub win_rate: Option<Decimal>,
    pub total_trades: Option<i32>,
    pub total_return: Option<Decimal>,
    pub sharpe_ratio: Option<Decimal>,
    pub max_drawdown: Option<Decimal>,
    pub profit_factor: Option<Decimal>,
    pub training_duration_seconds: Option<Decimal>,
    pub backtest_duration_seconds: Option<Decimal>,
    pub model_file_path: Option<String>,
    pub rejection_reason: Option<String>,
    pub created_at: Option<DateTime>,
    pub wf_median_win_rate: Option<Decimal>,
    pub wf_mean_return: Option<Decimal>,
    pub wf_max_drawdown_worst: Option<Decimal>,
    pub wf_passed_gates: Option<bool>,
    pub walkforward_duration_seconds: Option<Decimal>,
    pub oos_win_rate: Option<Decimal>,
    pub oos_total_trades: Option<i32>,
    pub oos_return: Option<Decimal>,
    pub oos_max_drawdown: Option<Decimal>,
    pub oos_passed_gates: Option<bool>,
    pub oos_duration_seconds: Option<Decimal>,
    pub optimization_duration_seconds: Option<Decimal>,
    pub interval: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
