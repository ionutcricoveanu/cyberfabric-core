use sea_orm::entity::prelude::*;
use modkit_db_macros::Scopable;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Scopable)]
#[sea_orm(table_name = "agent_trade_impact")]
#[secure(unrestricted)]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub pair: String,
    pub internal_id: Option<String>,
    pub agent_recommendation: Option<String>,
    pub recommendation_confidence: Option<f64>,
    pub action_taken: Option<String>,
    pub entry_price: Option<Decimal>,
    pub exit_price: Option<Decimal>,
    pub exit_time: Option<DateTime>,
    pub profit_pct: Option<f64>,
    pub sentiment_at_entry: Option<f64>,
    pub sentiment_at_exit: Option<f64>,
    pub counterfactual_profit_pct: Option<f64>,
    pub created_at: Option<DateTime>,
    pub environment: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
