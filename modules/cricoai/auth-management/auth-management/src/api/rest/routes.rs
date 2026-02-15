use std::sync::Arc;

use axum::Router;
use modkit::api::OpenApiRegistry;
use modkit::api::operation_builder::{AuthReqAction, AuthReqResource, LicenseFeature, OperationBuilder};
use tracing::info;

use super::dto;
use super::handlers;
use crate::domain::service::AuthManagementService;

// ── Local auth enums ─────────

pub(crate) enum Resource { Auth }

impl AsRef<str> for Resource {
    fn as_ref(&self) -> &str {
        match self { Self::Auth => "auth" }
    }
}

impl AuthReqResource for Resource {}

pub(crate) enum Action { Read, Write }

impl AsRef<str> for Action {
    fn as_ref(&self) -> &str {
        match self {
            Self::Read => "read",
            Self::Write => "write",
        }
    }
}

impl AuthReqAction for Action {}

pub(crate) struct License;

impl AsRef<str> for License {
    fn as_ref(&self) -> &str { "" }
}

impl LicenseFeature for License {}

pub fn register_routes(
    mut router: Router,
    openapi: &dyn OpenApiRegistry,
    service: Arc<AuthManagementService>,
) -> Router {
    info!("Registering auth-management REST routes");

    // ── User Management ──

    router = OperationBuilder::get("/auth-management/v1/users")
        .operation_id("auth_management.get_users")
        .summary("Get all users")
        .description("Get all users with their roles (admin only)")
        .tag("auth-management")
        .require_auth(&Resource::Auth, &Action::Read)
        .require_license_features::<License>([])
        .handler(handlers::get_users)
        .json_response_with_schema::<dto::UsersResponse>(openapi, http::StatusCode::OK, "Users list")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    router = OperationBuilder::post("/auth-management/v1/users")
        .operation_id("auth_management.create_user")
        .summary("Create user")
        .description("Create a new user (admin only)")
        .tag("auth-management")
        .require_auth(&Resource::Auth, &Action::Write)
        .require_license_features::<License>([])
        .handler(handlers::create_user)
        .json_response_with_schema::<dto::UserCreatedResponse>(openapi, http::StatusCode::OK, "User created")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    router = OperationBuilder::put("/auth-management/v1/users/{user_id}")
        .operation_id("auth_management.update_user")
        .summary("Update user")
        .description("Update user information (admin only)")
        .tag("auth-management")
        .require_auth(&Resource::Auth, &Action::Write)
        .require_license_features::<License>([])
        .handler(handlers::update_user)
        .json_response_with_schema::<dto::SuccessResponse>(openapi, http::StatusCode::OK, "User updated")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    router = OperationBuilder::delete("/auth-management/v1/users/{user_id}")
        .operation_id("auth_management.delete_user")
        .summary("Delete user")
        .description("Deactivate a user (admin only)")
        .tag("auth-management")
        .require_auth(&Resource::Auth, &Action::Write)
        .require_license_features::<License>([])
        .handler(handlers::delete_user)
        .json_response_with_schema::<dto::SuccessResponse>(openapi, http::StatusCode::OK, "User deleted")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    // ── Role Management ──

    router = OperationBuilder::post("/auth-management/v1/users/{user_id}/roles")
        .operation_id("auth_management.assign_role")
        .summary("Assign role")
        .description("Assign a role to a user (admin only)")
        .tag("auth-management")
        .require_auth(&Resource::Auth, &Action::Write)
        .require_license_features::<License>([])
        .handler(handlers::assign_role)
        .json_response_with_schema::<dto::SuccessResponse>(openapi, http::StatusCode::OK, "Role assigned")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    router = OperationBuilder::delete("/auth-management/v1/users/{user_id}/roles/{role_name}")
        .operation_id("auth_management.remove_role")
        .summary("Remove role")
        .description("Remove a role from a user (admin only)")
        .tag("auth-management")
        .require_auth(&Resource::Auth, &Action::Write)
        .require_license_features::<License>([])
        .handler(handlers::remove_role)
        .json_response_with_schema::<dto::SuccessResponse>(openapi, http::StatusCode::OK, "Role removed")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    // ── 2FA Management ──

    router = OperationBuilder::post("/auth-management/v1/users/{user_id}/2fa/setup")
        .operation_id("auth_management.setup_2fa")
        .summary("Setup 2FA")
        .description("Generate TOTP secret and QR code for 2FA setup")
        .tag("auth-management")
        .require_auth(&Resource::Auth, &Action::Write)
        .require_license_features::<License>([])
        .handler(handlers::setup_2fa)
        .json_response_with_schema::<dto::TwoFactorSetupResponse>(openapi, http::StatusCode::OK, "2FA setup")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    router = OperationBuilder::post("/auth-management/v1/users/{user_id}/2fa/verify")
        .operation_id("auth_management.verify_2fa")
        .summary("Verify 2FA")
        .description("Verify TOTP token and enable 2FA")
        .tag("auth-management")
        .require_auth(&Resource::Auth, &Action::Write)
        .require_license_features::<License>([])
        .handler(handlers::verify_2fa)
        .json_response_with_schema::<dto::SuccessResponse>(openapi, http::StatusCode::OK, "2FA verified")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    router = OperationBuilder::delete("/auth-management/v1/users/{user_id}/2fa")
        .operation_id("auth_management.disable_2fa")
        .summary("Disable 2FA")
        .description("Disable 2FA for a user (admin only)")
        .tag("auth-management")
        .require_auth(&Resource::Auth, &Action::Write)
        .require_license_features::<License>([])
        .handler(handlers::disable_2fa)
        .json_response_with_schema::<dto::SuccessResponse>(openapi, http::StatusCode::OK, "2FA disabled")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    // ── IP Management ──

    router = OperationBuilder::get("/auth-management/v1/security/ip-blocks")
        .operation_id("auth_management.get_ip_blocks")
        .summary("Get IP blocks")
        .description("Get IP blacklist and whitelist (admin only)")
        .tag("auth-management")
        .require_auth(&Resource::Auth, &Action::Read)
        .require_license_features::<License>([])
        .handler(handlers::get_ip_blocks)
        .json_response_with_schema::<dto::IpBlocksResponse>(openapi, http::StatusCode::OK, "IP blocks")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    router = OperationBuilder::post("/auth-management/v1/security/ip-blocks")
        .operation_id("auth_management.block_ip")
        .summary("Block IP")
        .description("Block an IP address (admin only)")
        .tag("auth-management")
        .require_auth(&Resource::Auth, &Action::Write)
        .require_license_features::<License>([])
        .handler(handlers::block_ip)
        .json_response_with_schema::<dto::IpBlockCreatedResponse>(openapi, http::StatusCode::OK, "IP blocked")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    router = OperationBuilder::delete("/auth-management/v1/security/ip-blocks/{block_id}")
        .operation_id("auth_management.unblock_ip")
        .summary("Unblock IP")
        .description("Remove an IP from the blacklist (admin only)")
        .tag("auth-management")
        .require_auth(&Resource::Auth, &Action::Write)
        .require_license_features::<License>([])
        .handler(handlers::unblock_ip)
        .json_response_with_schema::<dto::SuccessResponse>(openapi, http::StatusCode::OK, "IP unblocked")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    router = OperationBuilder::post("/auth-management/v1/security/ip-whitelist")
        .operation_id("auth_management.whitelist_ip")
        .summary("Whitelist IP")
        .description("Add IP to whitelist (admin only)")
        .tag("auth-management")
        .require_auth(&Resource::Auth, &Action::Write)
        .require_license_features::<License>([])
        .handler(handlers::whitelist_ip)
        .json_response_with_schema::<dto::IpWhitelistCreatedResponse>(openapi, http::StatusCode::OK, "IP whitelisted")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    router = OperationBuilder::delete("/auth-management/v1/security/ip-whitelist/{whitelist_id}")
        .operation_id("auth_management.remove_whitelist")
        .summary("Remove from whitelist")
        .description("Remove IP from whitelist (admin only)")
        .tag("auth-management")
        .require_auth(&Resource::Auth, &Action::Write)
        .require_license_features::<License>([])
        .handler(handlers::remove_whitelist)
        .json_response_with_schema::<dto::SuccessResponse>(openapi, http::StatusCode::OK, "Removed from whitelist")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    // ── Audit Logs ──

    router = OperationBuilder::get("/auth-management/v1/audit-logs")
        .operation_id("auth_management.get_audit_logs")
        .summary("Get audit logs")
        .description("Get audit logs with optional filters (admin only)")
        .tag("auth-management")
        .require_auth(&Resource::Auth, &Action::Read)
        .require_license_features::<License>([])
        .query_param("limit", false, "Maximum logs to return (default: 100)")
        .query_param("offset", false, "Offset for pagination")
        .query_param("user_id", false, "Filter by user ID")
        .query_param("action_type", false, "Filter by action type")
        .query_param("start_date", false, "Filter by start date")
        .query_param("end_date", false, "Filter by end date")
        .handler(handlers::get_audit_logs)
        .json_response_with_schema::<dto::AuditLogsResponse>(openapi, http::StatusCode::OK, "Audit logs")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    // ── Sessions (placeholder) ──

    router = OperationBuilder::get("/auth-management/v1/sessions")
        .operation_id("auth_management.get_sessions")
        .summary("Get sessions")
        .description("Get active sessions for the current user")
        .tag("auth-management")
        .require_auth(&Resource::Auth, &Action::Read)
        .require_license_features::<License>([])
        .handler(handlers::get_sessions)
        .json_response_with_schema::<dto::SessionsResponse>(openapi, http::StatusCode::OK, "Sessions")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    router = OperationBuilder::delete("/auth-management/v1/sessions/{session_id}")
        .operation_id("auth_management.revoke_session")
        .summary("Revoke session")
        .description("Revoke a specific session")
        .tag("auth-management")
        .require_auth(&Resource::Auth, &Action::Write)
        .require_license_features::<License>([])
        .handler(handlers::revoke_session)
        .json_response_with_schema::<dto::SuccessResponse>(openapi, http::StatusCode::OK, "Session revoked")
        .error_400(openapi).error_500(openapi)
        .register(router, openapi);

    info!("Auth-management REST routes registered successfully");

    router.layer(axum::Extension(service))
}
