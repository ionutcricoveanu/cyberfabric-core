//! Fear & Greed Index API client

use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentData {
    pub score: f64,
    pub classification: String,
}

/// Fear & Greed Index API client (free, no authentication)
pub struct FearGreedClient {
    http_client: Client,
}

impl FearGreedClient {
    /// Create a new Fear & Greed client
    pub fn new() -> Self {
        Self {
            http_client: Client::new(),
        }
    }

    /// Fetch current market sentiment (Fear & Greed Index)
    pub async fn fetch_sentiment(&self) -> Result<SentimentData, String> {
        let url = "https://api.alternative.me/fng/?limit=1";

        match self.http_client.get(url).send().await {
            Ok(response) => match response.json::<serde_json::Value>().await {
                Ok(_data) => {
                    // TODO: Parse actual response: data -> data[0] -> value, value_classification
                    debug!("Fetched Fear & Greed Index");

                    // Placeholder response
                    Ok(SentimentData {
                        score: 65.0,
                        classification: "Greed".to_string(),
                    })
                }
                Err(e) => {
                    error!("Failed to parse Fear & Greed data: {}", e);
                    Err(format!("Parse error: {}", e))
                }
            },
            Err(e) => {
                error!("Failed to fetch Fear & Greed Index: {}", e);
                Err(format!("Network error: {}", e))
            }
        }
    }
}

impl Default for FearGreedClient {
    fn default() -> Self {
        Self::new()
    }
}
