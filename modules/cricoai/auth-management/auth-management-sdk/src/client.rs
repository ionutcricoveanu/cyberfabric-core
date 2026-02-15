use async_trait::async_trait;
use modkit_security::SecurityContext;

use crate::errors::AuthManagementError;

#[async_trait]
pub trait AuthManagementApi: Send + Sync + 'static {
    async fn health_check(&self, ctx: &SecurityContext) -> Result<(), AuthManagementError>;
}
