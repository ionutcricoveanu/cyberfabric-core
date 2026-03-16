//! Data ingestion service - orchestrates all ingestion collectors

use std::sync::Arc;

use data_ingestion_sdk::{DataIngestionError, SentimentEntry};
use modkit_db::Db;
use modkit_security::SecurityContext;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info};

use crate::config::DataIngestionConfig;
use crate::infra::clients::{
    coingecko::CoinGeckoClient, fear_greed::FearGreedClient, reddit::RedditRSSClient,
};

/// Status of the collector
#[derive(Debug, Clone)]
pub struct CollectorStatus {
    pub running: bool,
    pub symbols_monitored: Vec<String>,
}

/// Data Ingestion Service - orchestrates multiple data sources
pub struct DataIngestionService {
    db: Option<Arc<Db>>,
    config: Option<DataIngestionConfig>,
    running: bool,
}

impl Default for DataIngestionService {
    fn default() -> Self {
        Self {
            db: None,
            config: None,
            running: false,
        }
    }
}

impl DataIngestionService {
    /// Create a new data ingestion service
    pub fn new(db: Arc<Db>,config: DataIngestionConfig) -> Self {
        Self {
            db: Some(db),
            config: Some(config),
            running: false,
        }
    }

    /// Get the latest sentiment entries for a symbol
    pub async fn get_latest_sentiment(
        &self,
        _ctx: &SecurityContext,
        symbol: &str,
    ) -> Result<Vec<SentimentEntry>, DataIngestionError> {
        // This will query the database for sentiment entries
        // For now, return empty vec - will be implemented when database schema is added
        debug!("Querying sentiment for symbol: {}", symbol);

        // TODO: Query market_sentiment table from Model_Data database
        Ok(Vec::new())
    }

    /// Get collector status
    pub fn get_status(&self) -> CollectorStatus {
        let config = self.config.as_ref();
        CollectorStatus {
            running: self.running,
            symbols_monitored: config
                .map(|c| c.symbols.clone())
                .unwrap_or_default(),
        }
    }

    /// Run the data collectors
    pub async fn run_collector(&self, cancellation: CancellationToken) -> anyhow::Result<()> {
        let config = self.config.as_ref().ok_or_else(|| {
            anyhow::anyhow!("Service not properly initialized - missing config")
        })?;

        info!("Starting data ingestion collectors");
        info!(
            "Monitoring symbols: {}",
            config.symbols.join(", ")
        );

        // Create collector clients
        let reddit_client = RedditRSSClient::new();
        let coingecko_client = CoinGeckoClient::new();
        let fear_greed_client = FearGreedClient::new();

        // Spawn parallel collector tasks
        let mut tasks: Vec<JoinHandle<()>> = Vec::new();

        // Reddit RSS collector (60 minutes default)
        let reddit_cfg = config.clone();
        let reddit_cancel = cancellation.clone();
        let reddit_task = tokio::spawn(async move {
            Self::run_reddit_rss_collector(&reddit_client, &reddit_cfg, reddit_cancel).await;
        });
        tasks.push(reddit_task);

        // CoinGecko collector (30 minutes default)
        let gecko_cfg = config.clone();
        let gecko_cancel = cancellation.clone();
        let gecko_task = tokio::spawn(async move {
            Self::run_coingecko_collector(&coingecko_client, &gecko_cfg, gecko_cancel).await;
        });
        tasks.push(gecko_task);

        // Fear & Greed Index collector (30 minutes default)
        let fear_cfg = config.clone();
        let fear_cancel = cancellation.clone();
        let fear_task = tokio::spawn(async move {
            Self::run_fear_greed_collector(&fear_greed_client, &fear_cfg, fear_cancel).await;
        });
        tasks.push(fear_task);

        info!(
            "Launched {} parallel collector tasks",
            tasks.len()
        );

        // Wait for cancellation
        cancellation.cancelled().await;
        info!("Collector cancellation signal received");

        // Cancel all tasks
        for task in tasks {
            task.abort();
        }

        info!("All collector tasks stopped");
        Ok(())
    }

    /// Reddit RSS collector loop
    async fn run_reddit_rss_collector(
        client: &RedditRSSClient,
        config: &DataIngestionConfig,
        cancellation: CancellationToken,
    ) {
        let interval_secs = config.reddit_rss_interval_minutes * 60;
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(interval_secs));

        loop {
            tokio::select! {
                _ = cancellation.cancelled() => {
                    info!("Reddit RSS collector cancelled");
                    break;
                }
                _ = interval.tick() => {
                    match client.fetch_posts(&config.symbols).await {
                        Ok(posts_count) => {
                            info!("Reddit RSS: fetched {} posts", posts_count);
                        }
                        Err(e) => {
                            error!("Reddit RSS collection error: {}", e);
                        }
                    }
                }
            }
        }
    }

    /// CoinGecko collector loop
    async fn run_coingecko_collector(
        client: &CoinGeckoClient,
        config: &DataIngestionConfig,
        cancellation: CancellationToken,
    ) {
        let interval_secs = config.coingecko_interval_minutes * 60;
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(interval_secs));

        loop {
            tokio::select! {
                _ = cancellation.cancelled() => {
                    info!("CoinGecko collector cancelled");
                    break;
                }
                _ = interval.tick() => {
                    match client.fetch_global_data().await {
                        Ok(_) => {
                            info!("CoinGecko: fetched global market data");
                        }
                        Err(e) => {
                            error!("CoinGecko collection error: {}", e);
                        }
                    }
                }
            }
        }
    }

    /// Fear & Greed Index collector loop
    async fn run_fear_greed_collector(
        client: &FearGreedClient,
        config: &DataIngestionConfig,
        cancellation: CancellationToken,
    ) {
        let interval_secs = config.sentiment_interval_minutes * 60;
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(interval_secs));

        loop {
            tokio::select! {
                _ = cancellation.cancelled() => {
                    info!("Fear & Greed collector cancelled");
                    break;
                }
                _ = interval.tick() => {
                    match client.fetch_sentiment().await {
                        Ok(sentiment) => {
                            info!(
                                "Fear & Greed: {} ({})",
                                sentiment.score,
                                sentiment.classification
                            );
                        }
                        Err(e) => {
                            error!("Fear & Greed collection error: {}", e);
                        }
                    }
                }
            }
        }
    }
}
