//! Local client adapter for in-process communication via ClientHub

use async_trait::async_trait;
use data_ingestion_sdk::{DataIngestionApi, DataIngestionError, SentimentEntry};
use modkit_security::SecurityContext;
use std::sync::Arc;

use crate::domain::service::DataIngestionService;

/// Local client implementing the DataIngestionApi trait
pub struct DataIngestionLocalClient {
    service: Arc<DataIngestionService>,
}

impl DataIngestionLocalClient {
    /// Create a new local client
    pub fn new(service: Arc<DataIngestionService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl DataIngestionApi for DataIngestionLocalClient {
    async fn get_latest_sentiment(
        &self,
        ctx: &SecurityContext,
        symbol: &str,
    ) -> Result<Vec<SentimentEntry>, DataIngestionError> {
        self.service.get_latest_sentiment(ctx, symbol).await
    }
}
