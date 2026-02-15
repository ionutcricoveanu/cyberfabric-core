use std::sync::Arc;

use async_trait::async_trait;
use modkit_security::SecurityContext;

use auth_management_sdk::{AuthManagementApi, AuthManagementError};
use crate::domain::service::AuthManagementService;

pub struct AuthManagementLocalClient {
    service: Arc<AuthManagementService>,
}

impl AuthManagementLocalClient {
    pub fn new(service: Arc<AuthManagementService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl AuthManagementApi for AuthManagementLocalClient {
    async fn health_check(&self, ctx: &SecurityContext) -> Result<(), AuthManagementError> {
        self.service
            .get_users(ctx)
            .await
            .map_err(|e| AuthManagementError::Database(e.to_string()))?;
        Ok(())
    }
}
