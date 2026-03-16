use sea_orm::entity::prelude::*;
use modkit_db_macros::Scopable;

/// Entity for the trade_pairs table in the Binance database.
/// This is an unrestricted entity (no tenant scoping).
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Scopable)]
#[sea_orm(table_name = "trade_pairs")]
#[secure(unrestricted)]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub pair: String,
    pub active: Option<bool>,
    pub symbol: Option<String>,
    pub base_asset: Option<String>,
    pub quote_asset: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
