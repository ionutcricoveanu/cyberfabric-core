//! Object-safe client trait for the data-ingestion module.

use async_trait::async_trait;
use modkit_security::SecurityContext;

use crate::errors::DataIngestionError;
use crate::models::SentimentEntry;

#[async_trait]
pub trait DataIngestionApi: Send + Sync {
    /// Get latest raw sentiment entries for a symbol.
    async fn get_latest_sentiment(
        &self,
        ctx: &SecurityContext,
        symbol: &str,
    ) -> Result<Vec<SentimentEntry>, DataIngestionError>;
}
