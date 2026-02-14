//! Domain models for the auth-manager module.
//!
//! Transport-agnostic (no serde, no HTTP types).

use uuid::Uuid;

/// A platform user.
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub roles: Vec<String>,
    pub two_fa_enabled: bool,
}
