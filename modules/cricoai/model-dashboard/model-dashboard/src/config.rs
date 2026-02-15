use serde::Deserialize;

/// Module-specific configuration loaded from `cricoai.yaml`.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelDashboardConfig {
    /// DSN for the Model_Data database (raw SeaORM connection for dynamic queries).
    pub model_data_dsn: Option<String>,
}
