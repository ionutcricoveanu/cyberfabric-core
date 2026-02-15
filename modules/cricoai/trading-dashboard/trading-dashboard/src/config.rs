/// Configuration for the trading-dashboard module.
///
/// Loaded via `ctx.config::<TradingDashboardConfig>()` from YAML.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct TradingDashboardConfig {
    pub default_page_size: u64,
    pub max_page_size: u64,
    /// DSN for the Model_Data database (agents, models, predictions).
    /// Example: `postgresql://binance:password@localhost:5432/Model_Data`
    pub model_data_dsn: Option<String>,
}

impl Default for TradingDashboardConfig {
    fn default() -> Self {
        Self {
            default_page_size: 25,
            max_page_size: 200,
            model_data_dsn: None,
        }
    }
}
