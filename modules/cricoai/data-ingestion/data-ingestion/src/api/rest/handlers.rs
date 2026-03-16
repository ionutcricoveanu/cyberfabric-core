//! REST API handlers

use axum::extract::Path;
use axum::Extension;
use modkit::api::prelude::*;
use modkit_security::SecurityContext;
use std::sync::Arc;

use crate::domain::service::DataIngestionService;

use super::dto::*;
use super::error;

/// Get latest sentiment entries for a symbol
pub async fn get_latest_sentiment(
    Extension(ctx): Extension<SecurityContext>,
    Extension(service): Extension<Arc<DataIngestionService>>,
    Path(symbol): Path<String>,
) -> ApiResult<JsonBody<LatestSentimentsResponse>> {
    let sentiments = service
        .get_latest_sentiment(&ctx, &symbol)
        .await
        .map_err(error::to_problem)?;

    let dto_sentiments = sentiments
        .into_iter()
        .map(|s| SentimentEntryDto {
            source: s.source,
            symbol: s.symbol,
            sentiment_score: s.sentiment_score,
            title: s.title,
            url: s.url,
            collected_at: s.collected_at.to_string(),
        })
        .collect();

    Ok(Json(LatestSentimentsResponse {
        sentiments: dto_sentiments,
    }))
}

/// Get collector status
pub async fn get_status(
    Extension(service): Extension<Arc<DataIngestionService>>,
) -> ApiResult<JsonBody<CollectorStatusResponse>> {
    let status = service.get_status();

    Ok(Json(CollectorStatusResponse {
        running: status.running,
        symbols_monitored: status.symbols_monitored,
    }))
}
