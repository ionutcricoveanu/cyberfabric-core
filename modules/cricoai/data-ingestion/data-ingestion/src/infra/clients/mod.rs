//! External API clients for data sources

pub mod coingecko;
pub mod fear_greed;
pub mod reddit;

pub use coingecko::CoinGeckoClient;
pub use fear_greed::FearGreedClient;
pub use reddit::RedditRSSClient;
