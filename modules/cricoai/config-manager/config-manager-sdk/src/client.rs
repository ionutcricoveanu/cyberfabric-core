//! Object-safe client trait for the config-manager module.

use async_trait::async_trait;
use modkit_security::SecurityContext;

use crate::errors::ConfigManagerError;
use crate::models::PairConfig;

#[async_trait]
pub trait ConfigManagerApi: Send + Sync {
    /// Get configuration for a specific trading pair.
    async fn get_pair_config(
        &self,
        ctx: &SecurityContext,
        symbol: &str,
    ) -> Result<PairConfig, ConfigManagerError>;
}
