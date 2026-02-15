use sea_orm::entity::prelude::*;
use modkit_db_macros::Scopable;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Scopable)]
#[sea_orm(table_name = "trade_pairs")]
#[secure(unrestricted)]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub pair: Option<String>,
    pub open_orders: Option<i32>,
    pub high_price_hour: Option<f64>,
    pub low_price_hour: Option<f64>,
    pub high_price_day: Option<f64>,
    pub low_price_day: Option<f64>,
    pub midpoint: Option<f64>,
    pub trailing_percent: Option<f64>,
    pub buy_open_orders: Option<i32>,
    pub sell_open_orders: Option<i32>,
    pub midpoint_hour: Option<f64>,
    pub trailing_percent_hour: Option<f64>,
    pub start_price_hour: Option<f64>,
    pub start_price_day: Option<f64>,
    pub end_price_hour: Option<f64>,
    pub end_price_day: Option<f64>,
    pub signals_percent_difference: Option<f64>,
    pub last_purchase_time: Option<DateTime>,
    pub last_sell_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
