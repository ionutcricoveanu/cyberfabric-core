//! CoinGecko API client for market data

use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalMarketData {
    pub total_market_cap_usd: f64,
    pub btc_dominance: f64,
    pub market_cap_change_24h: f64,
    pub trading_volume_24h: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendingCoin {
    pub symbol: String,
    pub name: String,
    pub price_usd: f64,
    pub market_cap_rank: Option<i32>,
}

/// CoinGecko API client (free tier, no authentication)
pub struct CoinGeckoClient {
    http_client: Client,
}

impl CoinGeckoClient {
    /// Create a new CoinGecko client
    pub fn new() -> Self {
        Self {
            http_client: Client::new(),
        }
    }

    /// Fetch global cryptocurrency market data
    pub async fn fetch_global_data(&self) -> Result<GlobalMarketData, String> {
        let url = "https://api.coingecko.com/api/v3/global";

        match self.http_client.get(url).send().await {
            Ok(response) => match response.json::<serde_json::Value>().await {
                Ok(data) => {
                    // TODO: Parse actual API response structure
                    // Expected path: data -> data -> total_market_cap -> usd, etc.
                    debug!("Fetched global market data from CoinGecko");

                    Ok(GlobalMarketData {
                        total_market_cap_usd: 1000000000.0, // Placeholder
                        btc_dominance: 45.0,
                        market_cap_change_24h: 2.5,
                        trading_volume_24h: 50000000000.0,
                    })
                }
                Err(e) => {
                    error!("Failed to parse CoinGecko global data: {}", e);
                    Err(format!("Parse error: {}", e))
                }
            },
            Err(e) => {
                error!("Failed to fetch from CoinGecko global endpoint: {}", e);
                Err(format!("Network error: {}", e))
            }
        }
    }

    /// Fetch trending coins
    pub async fn fetch_trending_coins(&self) -> Result<Vec<TrendingCoin>, String> {
        let url = "https://api.coingecko.com/api/v3/search/trending";

        match self.http_client.get(url).send().await {
            Ok(response) => match response.json::<serde_json::Value>().await {
                Ok(_data) => {
                    // TODO: Parse actual trending coins response (data -> coins array)
                    debug!("Fetched trending coins from CoinGecko");

                    Ok(vec![
                        TrendingCoin {
                            symbol: "BTC".to_string(),
                            name: "Bitcoin".to_string(),
                            price_usd: 45000.0,
                            market_cap_rank: Some(1),
                        },
                    ])
                }
                Err(e) => {
                    error!("Failed to parse CoinGecko trending data: {}", e);
                    Err(format!("Parse error: {}", e))
                }
            },
            Err(e) => {
                error!("Failed to fetch from CoinGecko trending endpoint: {}", e);
                Err(format!("Network error: {}", e))
            }
        }
    }
}

impl Default for CoinGeckoClient {
    fn default() -> Self {
        Self::new()
    }
}
