use sea_orm::entity::prelude::*;
use modkit_db_macros::Scopable;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Scopable)]
#[sea_orm(table_name = "market_sentiment")]
#[secure(unrestricted)]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub symbol: String,
    pub sentiment_score: Option<f64>,
    pub sentiment_source: Option<String>,
    pub positive_signals: Option<Vec<String>>,
    pub negative_signals: Option<Vec<String>>,
    pub raw_data: Option<serde_json::Value>,
    pub created_at: Option<DateTime>,
    pub confidence: Option<f64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
