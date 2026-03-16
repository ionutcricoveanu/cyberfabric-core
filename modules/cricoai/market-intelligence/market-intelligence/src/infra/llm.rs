//! Ollama LLM client for market intelligence analysis

use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaRequest {
    pub model: String,
    pub prompt: String,
    pub stream: bool,
    pub temperature: f64,
    pub num_predict: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaResponse {
    pub response: String,
    pub created_at: String,
    pub model: String,
    pub done: bool,
    pub total_duration: u64,
    pub load_duration: u64,
    pub prompt_eval_count: u32,
    pub prompt_eval_duration: u64,
    pub eval_count: u32,
    pub eval_duration: u64,
}

/// Ollama LLM client for local model inference
pub struct OllamaClient {
    http_client: Client,
    base_url: String,
    model_name: String,
}

impl OllamaClient {
    /// Create a new Ollama client
    pub fn new(base_url: String, model_name: String) -> Self {
        Self {
            http_client: Client::new(),
            base_url,
            model_name,
        }
    }

    /// Generate a response from the LLM
    pub async fn generate_response(&self, prompt: &str) -> Result<String, String> {
        let url = format!("{}/api/generate", self.base_url);

        let request = OllamaRequest {
            model: self.model_name.clone(),
            prompt: prompt.to_string(),
            stream: false,
            temperature: 0.7,
            num_predict: 2048,
        };

        debug!(
            "Sending request to Ollama: {} (model: {})",
            self.base_url, self.model_name
        );

        match self.http_client.post(&url).json(&request).send().await {
            Ok(response) => match response.json::<OllamaResponse>().await {
                Ok(data) => {
                    debug!("Ollama response received: {} tokens generated", data.eval_count);
                    Ok(data.response)
                }
                Err(e) => {
                    error!("Failed to parse Ollama response: {}", e);
                    Err(format!("Parse error: {}", e))
                }
            },
            Err(e) => {
                error!("Failed to connect to Ollama at {}: {}", self.base_url, e);
                Err(format!("Connection error: {}", e))
            }
        }
    }

    /// Check if Ollama is reachable
    pub async fn health_check(&self) -> bool {
        let url = format!("{}/api/tags", self.base_url);

        match self.http_client.get(&url).send().await {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }
}
