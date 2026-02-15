use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthManagementConfig {
    /// DSN for the Binance database (auth tables live here).
    pub binance_dsn: Option<String>,
}
