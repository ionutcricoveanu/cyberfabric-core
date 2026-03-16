//! Reddit RSS feed client

use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

const REDDIT_RSS_SUBREDDITS: &[&str] = &[
    "cryptocurrency",
    "Bitcoin",
    "Ethereum",
    "CryptoCurrency",
    "defi",
    "NFTs",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedditPost {
    pub title: String,
    pub url: String,
    pub subreddit: String,
    pub score: i64,
    pub timestamp: i64,
}

/// Reddit RSS feed client (no authentication required)
pub struct RedditRSSClient {
    http_client: Client,
}

impl RedditRSSClient {
    /// Create a new Reddit RSS client
    pub fn new() -> Self {
        Self {
            http_client: Client::new(),
        }
    }

    /// Fetch posts from all subreddits
    pub async fn fetch_posts(&self, _symbols: &[String]) -> Result<usize, String> {
        let mut total_posts = 0;

        for subreddit in REDDIT_RSS_SUBREDDITS {
            let url = format!("https://www.reddit.com/r/{}.json?limit=25", subreddit);

            match self.http_client.get(&url).send().await {
                Ok(response) => match response.json::<serde_json::Value>().await {
                    Ok(_data) => {
                        // TODO: Parse actual Listing data structure
                        // For now, just counting successful fetches
                        total_posts += 5; // Placeholder
                        debug!(
                            "Fetched posts from r/{}: {}",
                            subreddit, total_posts
                        );
                    }
                    Err(e) => {
                        error!("Failed to parse Reddit JSON for r/{}: {}", subreddit, e);
                    }
                },
                Err(e) => {
                    error!("Failed to fetch from r/{}: {}", subreddit, e);
                }
            }
        }

        Ok(total_posts)
    }
}

impl Default for RedditRSSClient {
    fn default() -> Self {
        Self::new()
    }
}
