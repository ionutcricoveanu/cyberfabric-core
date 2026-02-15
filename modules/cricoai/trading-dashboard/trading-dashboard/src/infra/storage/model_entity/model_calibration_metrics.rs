use sea_orm::entity::prelude::*;
use modkit_db_macros::Scopable;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Scopable)]
#[sea_orm(table_name = "model_calibration_metrics")]
#[secure(unrestricted)]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub symbol: String,
    pub model_version: Option<String>,
    pub prediction_id: Option<i32>,
    pub prediction_timestamp: DateTime,
    pub confidence_score: Decimal,
    pub predicted_direction: String,
    pub actual_direction: Option<String>,
    pub direction_correct: Option<bool>,
    pub calibration_error: Option<Decimal>,
    pub market_regime: Option<String>,
    pub model_age_days: Option<i32>,
    pub regime_conflict: Option<bool>,
    pub regime_penalty: Option<Decimal>,
    pub age_penalty: Option<Decimal>,
    pub raw_confidence: Option<Decimal>,
    pub adjusted_confidence: Option<Decimal>,
    pub verification_timestamp: Option<DateTime>,
    pub created_at: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
