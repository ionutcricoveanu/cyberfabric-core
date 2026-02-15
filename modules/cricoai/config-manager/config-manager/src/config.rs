use serde::Deserialize;

/// Module-specific configuration loaded from `cricoai.yaml`.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigManagerConfig {
    /// Path to the production config YAML file.
    #[serde(default = "default_config_path")]
    pub config_path: String,

    /// Path to the testnet config YAML file.
    #[serde(default)]
    pub config_path_testnet: Option<String>,

    /// DSN for the Binance_Klines database (for signals endpoint).
    #[serde(default)]
    pub klines_dsn: Option<String>,
}

fn default_config_path() -> String {
    "/app/config.yml".to_string()
}
