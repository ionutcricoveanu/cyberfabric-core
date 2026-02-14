use sea_orm::entity::prelude::*;
use modkit_db_macros::Scopable;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Scopable)]
#[sea_orm(table_name = "temps")]
#[secure(unrestricted)]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i32,
    pub available_usdc: Option<f64>,
    pub estimated_pl: Option<i32>,
    pub not_enough: Option<i32>,
    pub print_assets: Option<i32>,
    pub stop_execution: Option<i32>,
    pub trading_enabled: Option<i32>,
    pub version_changed: Option<i32>,
    pub buy_orders_all_coins: Option<i32>,
    pub last_purchase_time_all_coins: Option<DateTime>,
    pub buy_orders_prices_increase: Option<i32>,
    pub total_value: Option<f64>,
    pub bnb: Option<f64>,
    pub stop_execution_kline: Option<i32>,
    pub last_integrity_check: Option<f64>,
    pub bnb_value: Option<f64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
