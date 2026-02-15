use sea_orm::entity::prelude::*;
use modkit_db_macros::Scopable;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Scopable)]
#[sea_orm(table_name = "df_view")]
#[secure(unrestricted)]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub pair: String,
    pub highest_price: Option<f64>,
    pub current_price: Option<f64>,
    pub lowes_price: Option<f64>,
    pub highest_percentage: Option<f64>,
    pub lowest_percentage: Option<f64>,
    pub one_day_change_percentage: Option<f64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
