use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentTradeImpactConfig {
    /// DSN for the Model_Data database.
    pub model_data_dsn: Option<String>,
}
