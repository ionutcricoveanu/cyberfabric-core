//! Object-safe client trait for the auth-manager module.

use async_trait::async_trait;
use modkit_security::SecurityContext;
use uuid::Uuid;

use crate::errors::AuthManagerError;
use crate::models::User;

#[async_trait]
pub trait AuthManagerApi: Send + Sync {
    /// Get a user by ID.
    async fn get_user(
        &self,
        ctx: &SecurityContext,
        id: Uuid,
    ) -> Result<Option<User>, AuthManagerError>;
}
