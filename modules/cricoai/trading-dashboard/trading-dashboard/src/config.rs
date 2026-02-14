/// Configuration for the trading-dashboard module.
///
/// Loaded via `ctx.config::<TradingDashboardConfig>()` from YAML.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct TradingDashboardConfig {
    pub default_page_size: u64,
    pub max_page_size: u64,
}

impl Default for TradingDashboardConfig {
    fn default() -> Self {
        Self {
            default_page_size: 25,
            max_page_size: 200,
        }
    }
}
