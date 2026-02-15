use serde::Deserialize;

// ================================================================
// User management DTOs
// ================================================================

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct UserDto {
    pub id: i64,
    pub username: String,
    pub email: Option<String>,
    pub full_name: Option<String>,
    pub is_active: bool,
    pub totp_enabled: bool,
    pub created_at: Option<String>,
    pub last_successful_login: Option<String>,
    pub roles: Vec<String>,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct UsersResponse {
    pub users: Vec<UserDto>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UserCreateRequest {
    pub username: String,
    pub password: String,
    pub email: Option<String>,
    pub full_name: Option<String>,
    #[serde(default)]
    pub roles: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UserUpdateRequest {
    pub email: Option<String>,
    pub full_name: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RoleAssignmentRequest {
    pub role_name: String,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct SuccessResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct UserCreatedResponse {
    pub success: bool,
    pub user_id: i64,
}

// ================================================================
// 2FA DTOs
// ================================================================

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct TwoFactorSetupResponse {
    pub success: bool,
    pub secret: String,
    pub qr_code: String,
    pub uri: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TwoFactorVerifyRequest {
    pub token: String,
}

// ================================================================
// IP management DTOs
// ================================================================

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct BlacklistEntryDto {
    pub id: i64,
    pub ip_address: String,
    pub reason: Option<String>,
    pub is_permanent: bool,
    pub blocked_until: Option<String>,
    pub auto_blocked: bool,
    pub geo_country: Option<String>,
    pub geo_city: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct WhitelistEntryDto {
    pub id: i64,
    pub ip_address: String,
    pub description: Option<String>,
    pub expires_at: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct IpBlocksResponse {
    pub blacklist: Vec<BlacklistEntryDto>,
    pub whitelist: Vec<WhitelistEntryDto>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IpBlockRequest {
    pub ip: String,
    pub reason: String,
    #[serde(default)]
    pub is_permanent: bool,
    pub blocked_until: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IpWhitelistRequest {
    pub ip: String,
    pub description: String,
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct IpBlockCreatedResponse {
    pub success: bool,
    pub block_id: i64,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct IpWhitelistCreatedResponse {
    pub success: bool,
    pub whitelist_id: i64,
}

// ================================================================
// Audit log DTOs
// ================================================================

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct AuditLogEntryDto {
    pub id: i64,
    pub user_id: Option<i64>,
    pub username: Option<String>,
    pub action_type: String,
    pub ip_address: Option<String>,
    pub details: Option<String>,
    pub severity: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct AuditLogsResponse {
    pub logs: Vec<AuditLogEntryDto>,
}

// ================================================================
// Session DTOs
// ================================================================

#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct SessionsResponse {
    pub sessions: Vec<serde_json::Value>,
    pub message: String,
}
