use sea_orm::entity::prelude::*;
use modkit_db_macros::Scopable;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Scopable)]
#[sea_orm(table_name = "sell_orders")]
#[secure(unrestricted)]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub symbol: Option<String>,
    pub order_id: Option<Decimal>,
    pub client_order_id: Option<String>,
    pub transact_time: Option<DateTime>,
    pub orig_qty: Option<f64>,
    pub executed_qty: Option<f64>,
    pub cummulative_quote_qty: Option<f64>,
    pub status: Option<String>,
    pub time_in_force: Option<String>,
    pub r#type: Option<String>,
    pub side: Option<String>,
    pub price: Option<f64>,
    pub qty: Option<f64>,
    pub commission: Option<f64>,
    pub commission_asset: Option<String>,
    pub trade_id: Option<Decimal>,
    pub trailing_delta: Option<f64>,
    pub trailing_time: Option<DateTime>,
    pub pnl: Option<f64>,
    pub order_price: Option<f64>,
    pub stop_price: Option<f64>,
    pub buy_order_id: Option<Decimal>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
