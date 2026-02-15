use axum::Extension;
use axum::extract::{Path, Query};
use modkit::api::prelude::*;
use modkit_security::SecurityContext;
use serde::Deserialize;
use std::sync::Arc;

use crate::api::rest::dto::*;
use crate::domain::service::AuthManagementService;

#[derive(Debug, Deserialize)]
pub struct AuditLogQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    pub user_id: Option<i64>,
    pub action_type: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

fn default_limit() -> i64 { 100 }

// ── User Management ──

pub(crate) async fn get_users(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<AuthManagementService>>,
) -> ApiResult<JsonBody<UsersResponse>> {
    let resp = svc.get_users(&ctx).await?;
    Ok(axum::Json(resp))
}

pub(crate) async fn create_user(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<AuthManagementService>>,
    axum::Json(body): axum::Json<UserCreateRequest>,
) -> ApiResult<JsonBody<UserCreatedResponse>> {
    let resp = svc.create_user(&ctx, &body, None).await?;
    Ok(axum::Json(resp))
}

pub(crate) async fn update_user(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<AuthManagementService>>,
    Path(user_id): Path<i64>,
    axum::Json(body): axum::Json<UserUpdateRequest>,
) -> ApiResult<JsonBody<SuccessResponse>> {
    let resp = svc.update_user(&ctx, user_id, &body, None, None).await?;
    Ok(axum::Json(resp))
}

pub(crate) async fn delete_user(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<AuthManagementService>>,
    Path(user_id): Path<i64>,
) -> ApiResult<JsonBody<SuccessResponse>> {
    let resp = svc.delete_user(&ctx, user_id, None, None).await?;
    Ok(axum::Json(resp))
}

pub(crate) async fn assign_role(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<AuthManagementService>>,
    Path(user_id): Path<i64>,
    axum::Json(body): axum::Json<RoleAssignmentRequest>,
) -> ApiResult<JsonBody<SuccessResponse>> {
    let resp = svc.assign_role(&ctx, user_id, &body.role_name, None, None).await?;
    Ok(axum::Json(resp))
}

pub(crate) async fn remove_role(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<AuthManagementService>>,
    Path((user_id, role_name)): Path<(i64, String)>,
) -> ApiResult<JsonBody<SuccessResponse>> {
    let resp = svc.remove_role(&ctx, user_id, &role_name, None, None).await?;
    Ok(axum::Json(resp))
}

// ── 2FA ──

pub(crate) async fn setup_2fa(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<AuthManagementService>>,
    Path(user_id): Path<i64>,
) -> ApiResult<JsonBody<TwoFactorSetupResponse>> {
    let resp = svc.setup_2fa(&ctx, user_id).await?;
    Ok(axum::Json(resp))
}

pub(crate) async fn verify_2fa(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<AuthManagementService>>,
    Path(user_id): Path<i64>,
    axum::Json(body): axum::Json<TwoFactorVerifyRequest>,
) -> ApiResult<JsonBody<SuccessResponse>> {
    let resp = svc.verify_2fa(&ctx, user_id, &body.token).await?;
    Ok(axum::Json(resp))
}

pub(crate) async fn disable_2fa(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<AuthManagementService>>,
    Path(user_id): Path<i64>,
) -> ApiResult<JsonBody<SuccessResponse>> {
    let resp = svc.disable_2fa(&ctx, user_id, None).await?;
    Ok(axum::Json(resp))
}

// ── IP Management ──

pub(crate) async fn get_ip_blocks(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<AuthManagementService>>,
) -> ApiResult<JsonBody<IpBlocksResponse>> {
    let resp = svc.get_ip_blocks(&ctx).await?;
    Ok(axum::Json(resp))
}

pub(crate) async fn block_ip(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<AuthManagementService>>,
    axum::Json(body): axum::Json<IpBlockRequest>,
) -> ApiResult<JsonBody<IpBlockCreatedResponse>> {
    let resp = svc.block_ip(&ctx, &body, None).await?;
    Ok(axum::Json(resp))
}

pub(crate) async fn unblock_ip(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<AuthManagementService>>,
    Path(block_id): Path<i64>,
) -> ApiResult<JsonBody<SuccessResponse>> {
    let resp = svc.unblock_ip(&ctx, block_id, None).await?;
    Ok(axum::Json(resp))
}

pub(crate) async fn whitelist_ip(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<AuthManagementService>>,
    axum::Json(body): axum::Json<IpWhitelistRequest>,
) -> ApiResult<JsonBody<IpWhitelistCreatedResponse>> {
    let resp = svc.whitelist_ip(&ctx, &body, None).await?;
    Ok(axum::Json(resp))
}

pub(crate) async fn remove_whitelist(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<AuthManagementService>>,
    Path(whitelist_id): Path<i64>,
) -> ApiResult<JsonBody<SuccessResponse>> {
    let resp = svc.remove_whitelist(&ctx, whitelist_id, None).await?;
    Ok(axum::Json(resp))
}

// ── Audit Logs ──

pub(crate) async fn get_audit_logs(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<AuthManagementService>>,
    Query(q): Query<AuditLogQuery>,
) -> ApiResult<JsonBody<AuditLogsResponse>> {
    let limit = q.limit.min(500).max(1);
    let offset = q.offset.max(0);
    let resp = svc.get_audit_logs(
        &ctx, limit, offset,
        q.user_id,
        q.action_type.as_deref(),
        q.start_date.as_deref(),
        q.end_date.as_deref(),
    ).await?;
    Ok(axum::Json(resp))
}

// ── Sessions (placeholder) ──

pub(crate) async fn get_sessions(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<AuthManagementService>>,
) -> ApiResult<JsonBody<SessionsResponse>> {
    let resp = svc.get_sessions(&ctx).await?;
    Ok(axum::Json(resp))
}

pub(crate) async fn revoke_session(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<AuthManagementService>>,
    Path(session_id): Path<i64>,
) -> ApiResult<JsonBody<SuccessResponse>> {
    let resp = svc.revoke_session(&ctx, session_id).await?;
    Ok(axum::Json(resp))
}
