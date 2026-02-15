use sea_orm::entity::prelude::*;
use modkit_db_macros::Scopable;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Scopable)]
#[sea_orm(table_name = "agent_config")]
#[secure(unrestricted)]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub agent_type: String,
    pub enabled: Option<bool>,
    pub mode: Option<String>,
    pub decision_frequency_minutes: Option<i32>,
    pub confidence_threshold: Option<f64>,
    pub config: Option<serde_json::Value>,
    pub llm_provider: Option<String>,
    pub llm_model: Option<String>,
    pub updated_at: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
