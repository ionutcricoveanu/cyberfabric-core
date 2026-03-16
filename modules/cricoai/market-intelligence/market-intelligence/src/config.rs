//! Configuration for the market-intelligence module

use serde::{Deserialize, Serialize};

/// Tier-based scheduling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierScheduleConfig {
    /// HOT tier: Actively traded symbols (5 minutes default)
    pub hot_interval_minutes: u64,

    /// WARM tier: Medium activity (15 minutes default)
    pub warm_interval_minutes: u64,

    /// STABLE tier: Slower moving symbols (30 minutes default)
    pub stable_interval_minutes: u64,

    /// COLD tier: Low activity / monitoring (1 hour default)
    pub cold_interval_minutes: u64,
}

impl Default for TierScheduleConfig {
    fn default() -> Self {
        Self {
            hot_interval_minutes: 5,
            warm_interval_minutes: 15,
            stable_interval_minutes: 30,
            cold_interval_minutes: 60,
        }
    }
}

/// LLM configuration (Ollama for local development)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// Ollama API endpoint (default: http://localhost:11434)
    #[serde(default = "default_ollama_url")]
    pub ollama_url: String,

    /// Model name to use (default: mistral)
    #[serde(default = "default_model_name")]
    pub model_name: String,

    /// LLM response timeout in seconds (default: 30)
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,

    /// Temperature for LLM responses (0-1, default: 0.7)
    #[serde(default = "default_temperature")]
    pub temperature: f64,

    /// Max tokens for LLM responses (default: 2048)
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
}

fn default_ollama_url() -> String {
    "http://localhost:11434".to_string()
}

fn default_model_name() -> String {
    "mistral".to_string()
}

fn default_timeout() -> u64 {
    30
}

fn default_temperature() -> f64 {
    0.7
}

fn default_max_tokens() -> u32 {
    2048
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            ollama_url: default_ollama_url(),
            model_name: default_model_name(),
            timeout_secs: default_timeout(),
            temperature: default_temperature(),
            max_tokens: default_max_tokens(),
        }
    }
}

/// Configuration for the market-intelligence module
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MarketIntelligenceConfig {
    /// PostgreSQL DSN for the Model_Data database
    pub model_data_dsn: String,

    /// Tier-based scheduling
    #[serde(default)]
    pub tier_schedule: TierScheduleConfig,

    /// LLM configuration
    #[serde(default)]
    pub llm: LlmConfig,

    /// List of symbols to monitor
    #[serde(default = "default_symbols")]
    pub symbols: Vec<String>,

    /// Enable market sentiment lookback (hours)
    #[serde(default = "default_lookback_hours")]
    pub sentiment_lookback_hours: u64,
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

fn default_lookback_hours() -> u64 {
    6
}

impl Default for MarketIntelligenceConfig {
    fn default() -> Self {
        Self {
            model_data_dsn: String::new(),
            tier_schedule: TierScheduleConfig::default(),
            llm: LlmConfig::default(),
            symbols: default_symbols(),
            sentiment_lookback_hours: default_lookback_hours(),
        }
    }
}
