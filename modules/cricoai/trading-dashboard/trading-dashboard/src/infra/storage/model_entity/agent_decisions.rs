use sea_orm::entity::prelude::*;
use modkit_db_macros::Scopable;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Scopable)]
#[sea_orm(table_name = "agent_decisions")]
#[secure(unrestricted)]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub agent_type: String,
    pub decision_type: String,
    pub target_service: Option<String>,
    pub symbol: Option<String>,
    pub recommendation: Option<String>,
    pub reasoning: Option<Vec<String>>,
    pub confidence: Option<f64>,
    pub action_data: Option<serde_json::Value>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub created_at: Option<DateTime>,
    pub processed_at: Option<DateTime>,
    pub llm_model: Option<String>,
    pub processing_time_ms: Option<i32>,
    pub cost_usd: Option<Decimal>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
