use sea_orm::entity::prelude::*;
use modkit_db_macros::Scopable;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Scopable)]
#[sea_orm(table_name = "agent_performance")]
#[secure(unrestricted)]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub agent_type: String,
    pub date: Date,
    pub decisions_made: Option<i32>,
    pub decisions_applied: Option<i32>,
    pub decisions_rejected: Option<i32>,
    pub avg_confidence: Option<f64>,
    pub avg_processing_time_ms: Option<i32>,
    pub total_cost_usd: Option<Decimal>,
    pub positive_outcomes: Option<i32>,
    pub negative_outcomes: Option<i32>,
    pub neutral_outcomes: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
