use sea_orm::entity::prelude::*;
use modkit_db_macros::Scopable;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Scopable)]
#[sea_orm(table_name = "pairs_exclusion_list")]
#[secure(unrestricted)]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub pair: Option<String>,
    pub added_date: Option<DateTime>,
    pub auto_remove: Option<bool>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
