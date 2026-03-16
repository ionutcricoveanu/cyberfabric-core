//! Configuration for the data-ingestion module

use serde::{Deserialize, Serialize};

/// Configuration for the data-ingestion module
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DataIngestionConfig {
    /// PostgreSQL DSN for the Model_Data database (where sentiment data is stored)
    pub model_data_dsn: String,

    /// List of symbols to monitor (e.g., ["BTC", "ETH", "BNB"])
    #[serde(default = "default_symbols")]
    pub symbols: Vec<String>,

    /// Reddit RSS ingestion interval in minutes (default: 60)
    #[serde(default = "default_reddit_interval")]
    pub reddit_rss_interval_minutes: u64,

    /// CoinGecko ingestion interval in minutes (default: 30)
    #[serde(default = "default_coingecko_interval")]
    pub coingecko_interval_minutes: u64,

    /// Fear & Greed Index interval in minutes (default: 30)
    #[serde(default = "default_sentiment_interval")]
    pub sentiment_interval_minutes: u64,

    /// On-chain data ingestion interval in minutes (default: 60)
    #[serde(default = "default_onchain_interval")]
    pub onchain_interval_minutes: u64,

    /// CryptoPanic API key (optional)
    #[serde(default)]
    pub cryptopanic_api_key: Option<String>,

    /// News API key (optional)
    #[serde(default)]
    pub newsapi_api_key: Option<String>,

    /// Dune Analytics API key (optional)
    #[serde(default)]
    pub dune_api_key: Option<String>,

    /// Glassnode API key (optional)
    #[serde(default)]
    pub glassnode_api_key: Option<String>,

    /// Moralis API key (optional)
    #[serde(default)]
    pub moralis_api_key: Option<String>,
}

fn default_symbols() -> Vec<String> {
    vec![
        "BTC".to_string(),
        "ETH".to_string(),
        "BNB".to_string(),
        "ADA".to_string(),
        "SOL".to_string(),
        "DOT".to_string(),
        "LINK".to_string(),
        "UNI".to_string(),
    ]
}

fn default_reddit_interval() -> u64 {
    60
}

fn default_coingecko_interval() -> u64 {
    30
}

fn default_sentiment_interval() -> u64 {
    30
}

fn default_onchain_interval() -> u64 {
    60
}

impl Default for DataIngestionConfig {
    fn default() -> Self {
        Self {
            model_data_dsn: String::new(),
            symbols: default_symbols(),
            reddit_rss_interval_minutes: default_reddit_interval(),
            coingecko_interval_minutes: default_coingecko_interval(),
            sentiment_interval_minutes: default_sentiment_interval(),
            onchain_interval_minutes: default_onchain_interval(),
            cryptopanic_api_key: None,
            newsapi_api_key: None,
            dune_api_key: None,
            glassnode_api_key: None,
            moralis_api_key: None,
        }
    }
}
