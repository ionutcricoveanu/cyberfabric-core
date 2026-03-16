//! Request/Response DTOs

use modkit_macros::api_dto;

/// Request to get latest sentiment for a symbol
#[api_dto(request)]
#[derive(Debug, Clone)]
pub struct GetLatestSentimentRequest {
    pub symbol: String,
}

/// Response: Sentiment entry
#[api_dto(response)]
#[derive(Debug, Clone)]
pub struct SentimentEntryDto {
    pub source: String,
    pub symbol: String,
    pub sentiment_score: f64,
    pub title: Option<String>,
    pub url: Option<String>,
    pub collected_at: String,
}

/// Response: List of sentiment entries
#[api_dto(response)]
#[derive(Debug, Clone)]
pub struct LatestSentimentsResponse {
    pub sentiments: Vec<SentimentEntryDto>,
}

/// Response: Collector status
#[api_dto(response)]
#[derive(Debug, Clone)]
pub struct CollectorStatusResponse {
    pub running: bool,
    pub symbols_monitored: Vec<String>,
}
