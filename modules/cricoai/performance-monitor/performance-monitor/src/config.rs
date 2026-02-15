use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PerformanceMonitorConfig {
    /// DSN for the Binance database (production).
    pub binance_dsn: Option<String>,
    /// DSN for the BinanceTN database (testnet).
    pub binance_tn_dsn: Option<String>,
}
