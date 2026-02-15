/// Placeholder for SDK-level domain models.
/// The model-dashboard is read-only; most types live in the impl crate DTOs.
pub struct ModelSummary {
    pub total_trainings: i64,
    pub total_promoted: i64,
    pub unique_symbols: i64,
    pub overall_promotion_rate: f64,
}
