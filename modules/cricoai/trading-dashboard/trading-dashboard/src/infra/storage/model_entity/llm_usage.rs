use sea_orm::entity::prelude::*;
use modkit_db_macros::Scopable;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Scopable)]
#[sea_orm(table_name = "llm_usage")]
#[secure(unrestricted)]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub agent_type: String,
    pub llm_provider: String,
    pub llm_model: String,
    pub input_tokens: Option<i32>,
    pub output_tokens: Option<i32>,
    pub processing_time_ms: Option<i32>,
    pub cost_usd: Option<Decimal>,
    pub request_type: Option<String>,
    pub request_summary: Option<String>,
    pub created_at: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
