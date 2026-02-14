//! Domain models for the data-ingestion module.
//!
//! Transport-agnostic (no serde, no HTTP types).

/// A raw sentiment data point from an external source.
pub struct SentimentEntry {
    pub source: String,
    pub symbol: String,
    pub sentiment_score: f64,
    pub title: Option<String>,
    pub url: Option<String>,
    pub collected_at: chrono::NaiveDateTime,
}
