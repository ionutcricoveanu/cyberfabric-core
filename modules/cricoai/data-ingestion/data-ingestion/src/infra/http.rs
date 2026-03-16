//! HTTP client utilities

use reqwest::{Client, ClientBuilder};
use std::time::Duration;

/// Create a configured HTTP client for external API calls
pub fn create_http_client() -> Client {
    ClientBuilder::new()
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .user_agent("CricoAI/1.0")
        .build()
        .expect("Failed to build HTTP client")
}
