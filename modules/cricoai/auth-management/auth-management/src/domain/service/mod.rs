use sea_orm::{ConnectionTrait, DatabaseConnection};
use rust_decimal::Decimal;
use modkit_security::SecurityContext;

use crate::config::AuthManagementConfig;
use crate::api::rest::dto::*;
use crate::domain::error::DomainError;

fn db_err(e: impl std::fmt::Display) -> DomainError {
    DomainError::Database(e.to_string())
}

fn opt_i64(row: &sea_orm::QueryResult, idx: usize) -> i64 {
    row.try_get_by_index::<i64>(idx)
        .or_else(|_| row.try_get_by_index::<i32>(idx).map(|v| v as i64))
        .or_else(|_| {
            row.try_get_by_index::<Decimal>(idx)
                .map(|d| d.to_string().parse::<i64>().unwrap_or(0))
        })
        .unwrap_or(0)
}

fn opt_i64_nullable(row: &sea_orm::QueryResult, idx: usize) -> Option<i64> {
    row.try_get_by_index::<i64>(idx)
        .ok()
        .or_else(|| row.try_get_by_index::<i32>(idx).ok().map(|v| v as i64))
}

fn opt_str(row: &sea_orm::QueryResult, idx: usize) -> Option<String> {
    row.try_get_by_index::<String>(idx).ok()
}

fn opt_bool(row: &sea_orm::QueryResult, idx: usize) -> bool {
    row.try_get_by_index::<bool>(idx).unwrap_or(false)
}

fn opt_ts(row: &sea_orm::QueryResult, idx: usize) -> Option<String> {
    row.try_get_by_index::<chrono::NaiveDateTime>(idx)
        .ok()
        .map(|dt| format!("{}Z", dt.format("%Y-%m-%dT%H:%M:%S")))
}

/// Parse a PG text[] array from a query result
fn parse_pg_array(row: &sea_orm::QueryResult, idx: usize) -> Vec<String> {
    // Try reading as Vec<String> first (sqlx supports this for text[])
    if let Ok(v) = row.try_get_by_index::<Vec<String>>(idx) {
        return v;
    }
    // Fallback: try reading as a PG text representation "{val1,val2}"
    if let Ok(s) = row.try_get_by_index::<String>(idx) {
        let trimmed = s.trim_matches(|c| c == '{' || c == '}');
        if trimmed.is_empty() || trimmed == "NULL" {
            return vec![];
        }
        return trimmed.split(',').map(|s| s.trim().to_string()).collect();
    }
    vec![]
}

pub struct AuthManagementService {
    db: DatabaseConnection,
    _config: AuthManagementConfig,
}

impl AuthManagementService {
    pub fn new(db: DatabaseConnection, config: AuthManagementConfig) -> Self {
        Self { db, _config: config }
    }

    // ================================================================
    // GET /auth-management/v1/users
    // ================================================================
    pub async fn get_users(
        &self,
        _ctx: &SecurityContext,
    ) -> Result<UsersResponse, DomainError> {
        let backend = self.db.get_database_backend();

        let rows = self.db
            .query_all(sea_orm::Statement::from_string(backend, r#"
                SELECT u.id, u.username, u.email, u.full_name, u.is_active,
                       u.totp_enabled, u.created_at, u.last_successful_login,
                       ARRAY_AGG(r.name) FILTER (WHERE r.name IS NOT NULL) as roles
                FROM auth_users u
                LEFT JOIN auth_user_roles ur ON u.id = ur.user_id
                LEFT JOIN auth_roles r ON r.id = ur.role_id
                GROUP BY u.id
                ORDER BY u.created_at DESC
            "#.to_string()))
            .await
            .map_err(db_err)?;

        let users: Vec<UserDto> = rows.iter().map(|row| {
            UserDto {
                id: opt_i64(row, 0),
                username: opt_str(row, 1).unwrap_or_default(),
                email: opt_str(row, 2),
                full_name: opt_str(row, 3),
                is_active: opt_bool(row, 4),
                totp_enabled: opt_bool(row, 5),
                created_at: opt_ts(row, 6),
                last_successful_login: opt_ts(row, 7),
                roles: parse_pg_array(row, 8),
            }
        }).collect();

        Ok(UsersResponse { users })
    }

    // ================================================================
    // POST /auth-management/v1/users
    // ================================================================
    pub async fn create_user(
        &self,
        _ctx: &SecurityContext,
        req: &UserCreateRequest,
        admin_user_id: Option<i64>,
    ) -> Result<UserCreatedResponse, DomainError> {
        let backend = self.db.get_database_backend();

        // Hash password with bcrypt
        let password_hash = bcrypt::hash(&req.password, 10)
            .map_err(|e| DomainError::Database(format!("Password hashing failed: {e}")))?;

        // Insert user
        let user_rows = self.db
            .query_all(sea_orm::Statement::from_string(backend, format!(
                "INSERT INTO auth_users (username, password_hash, email, full_name, is_active) \
                 VALUES ('{}', '{}', {}, {}, TRUE) RETURNING id",
                req.username.replace('\'', "''"),
                password_hash.replace('\'', "''"),
                req.email.as_ref().map(|e| format!("'{}'", e.replace('\'', "''"))).unwrap_or_else(|| "NULL".to_string()),
                req.full_name.as_ref().map(|n| format!("'{}'", n.replace('\'', "''"))).unwrap_or_else(|| "NULL".to_string()),
            )))
            .await
            .map_err(db_err)?;

        let user_id = user_rows.first()
            .map(|r| opt_i64(r, 0))
            .ok_or_else(|| DomainError::Database("Failed to create user".to_string()))?;

        // Assign roles
        for role_name in &req.roles {
            let safe_role = role_name.replace('\'', "''");
            self.db
                .execute(sea_orm::Statement::from_string(backend, format!(
                    "INSERT INTO auth_user_roles (user_id, role_id, assigned_by, assigned_at) \
                     SELECT {user_id}, id, {}, CURRENT_TIMESTAMP FROM auth_roles WHERE name = '{safe_role}' \
                     ON CONFLICT (user_id, role_id) DO NOTHING",
                    admin_user_id.map(|id| id.to_string()).unwrap_or_else(|| "NULL".to_string()),
                )))
                .await
                .map_err(db_err)?;
        }

        // Audit log
        let admin_id_sql = admin_user_id.map(|id| id.to_string()).unwrap_or_else(|| "NULL".to_string());
        self.db
            .execute(sea_orm::Statement::from_string(backend, format!(
                "INSERT INTO auth_audit_log (user_id, action_type, details, severity) \
                 VALUES ({admin_id_sql}, 'user_created', '{{\"username\": \"{}\", \"user_id\": {user_id}}}', 'info')",
                req.username.replace('\'', "''").replace('"', "\\\""),
            )))
            .await
            .map_err(db_err)?;

        Ok(UserCreatedResponse { success: true, user_id })
    }

    // ================================================================
    // PUT /auth-management/v1/users/{user_id}
    // ================================================================
    pub async fn update_user(
        &self,
        _ctx: &SecurityContext,
        user_id: i64,
        req: &UserUpdateRequest,
        admin_user_id: Option<i64>,
        client_ip: Option<&str>,
    ) -> Result<SuccessResponse, DomainError> {
        let backend = self.db.get_database_backend();

        let mut set_parts: Vec<String> = Vec::new();

        if let Some(email) = &req.email {
            set_parts.push(format!("email = '{}'", email.replace('\'', "''")));
        }
        if let Some(name) = &req.full_name {
            set_parts.push(format!("full_name = '{}'", name.replace('\'', "''")));
        }
        if let Some(active) = req.is_active {
            set_parts.push(format!("is_active = {active}"));
        }

        if set_parts.is_empty() {
            return Ok(SuccessResponse { success: false, message: Some("No fields to update".to_string()) });
        }

        set_parts.push("updated_at = CURRENT_TIMESTAMP".to_string());

        self.db
            .execute(sea_orm::Statement::from_string(backend, format!(
                "UPDATE auth_users SET {} WHERE id = {user_id}",
                set_parts.join(", "),
            )))
            .await
            .map_err(db_err)?;

        // Audit log
        let admin_id_sql = admin_user_id.map(|id| id.to_string()).unwrap_or_else(|| "NULL".to_string());
        let ip_sql = client_ip.map(|ip| format!("'{}'", ip.replace('\'', "''"))).unwrap_or_else(|| "NULL".to_string());
        self.db
            .execute(sea_orm::Statement::from_string(backend, format!(
                "INSERT INTO auth_audit_log (user_id, action_type, ip_address, details, severity) \
                 VALUES ({admin_id_sql}, 'user_updated', {ip_sql}, '{{\"user_id\": {user_id}}}', 'info')",
            )))
            .await
            .map_err(db_err)?;

        Ok(SuccessResponse { success: true, message: None })
    }

    // ================================================================
    // DELETE /auth-management/v1/users/{user_id}
    // ================================================================
    pub async fn delete_user(
        &self,
        _ctx: &SecurityContext,
        user_id: i64,
        admin_user_id: Option<i64>,
        client_ip: Option<&str>,
    ) -> Result<SuccessResponse, DomainError> {
        let backend = self.db.get_database_backend();

        // Soft delete
        self.db
            .execute(sea_orm::Statement::from_string(backend, format!(
                "UPDATE auth_users SET is_active = FALSE, updated_at = CURRENT_TIMESTAMP WHERE id = {user_id}"
            )))
            .await
            .map_err(db_err)?;

        // Invalidate sessions
        self.db
            .execute(sea_orm::Statement::from_string(backend, format!(
                "UPDATE auth_sessions SET is_active = FALSE, invalidated_at = CURRENT_TIMESTAMP, \
                 invalidation_reason = 'user_deleted' WHERE user_id = {user_id} AND is_active = TRUE"
            )))
            .await
            .map_err(db_err)?;

        // Audit log
        let admin_id_sql = admin_user_id.map(|id| id.to_string()).unwrap_or_else(|| "NULL".to_string());
        let ip_sql = client_ip.map(|ip| format!("'{}'", ip.replace('\'', "''"))).unwrap_or_else(|| "NULL".to_string());
        self.db
            .execute(sea_orm::Statement::from_string(backend, format!(
                "INSERT INTO auth_audit_log (user_id, action_type, ip_address, details, severity) \
                 VALUES ({admin_id_sql}, 'user_deleted', {ip_sql}, '{{\"user_id\": {user_id}}}', 'warning')",
            )))
            .await
            .map_err(db_err)?;

        Ok(SuccessResponse { success: true, message: None })
    }

    // ================================================================
    // POST /auth-management/v1/users/{user_id}/roles
    // ================================================================
    pub async fn assign_role(
        &self,
        _ctx: &SecurityContext,
        user_id: i64,
        role_name: &str,
        admin_user_id: Option<i64>,
        client_ip: Option<&str>,
    ) -> Result<SuccessResponse, DomainError> {
        let backend = self.db.get_database_backend();
        let safe_role = role_name.replace('\'', "''");

        // Check role exists
        let role_rows = self.db
            .query_all(sea_orm::Statement::from_string(backend, format!(
                "SELECT id FROM auth_roles WHERE name = '{safe_role}'"
            )))
            .await
            .map_err(db_err)?;

        if role_rows.is_empty() {
            return Err(DomainError::NotFound(format!("Role '{}' not found", role_name)));
        }

        let role_id = opt_i64(&role_rows[0], 0);
        let admin_id_sql = admin_user_id.map(|id| id.to_string()).unwrap_or_else(|| "NULL".to_string());

        self.db
            .execute(sea_orm::Statement::from_string(backend, format!(
                "INSERT INTO auth_user_roles (user_id, role_id, assigned_by, assigned_at) \
                 VALUES ({user_id}, {role_id}, {admin_id_sql}, CURRENT_TIMESTAMP) \
                 ON CONFLICT (user_id, role_id) DO NOTHING"
            )))
            .await
            .map_err(db_err)?;

        // Audit log
        let ip_sql = client_ip.map(|ip| format!("'{}'", ip.replace('\'', "''"))).unwrap_or_else(|| "NULL".to_string());
        self.db
            .execute(sea_orm::Statement::from_string(backend, format!(
                "INSERT INTO auth_audit_log (user_id, action_type, ip_address, details, severity) \
                 VALUES ({admin_id_sql}, 'role_assigned', {ip_sql}, \
                 '{{\"user_id\": {user_id}, \"role\": \"{safe_role}\"}}', 'info')",
            )))
            .await
            .map_err(db_err)?;

        Ok(SuccessResponse { success: true, message: None })
    }

    // ================================================================
    // DELETE /auth-management/v1/users/{user_id}/roles/{role_name}
    // ================================================================
    pub async fn remove_role(
        &self,
        _ctx: &SecurityContext,
        user_id: i64,
        role_name: &str,
        admin_user_id: Option<i64>,
        client_ip: Option<&str>,
    ) -> Result<SuccessResponse, DomainError> {
        let backend = self.db.get_database_backend();
        let safe_role = role_name.replace('\'', "''");

        self.db
            .execute(sea_orm::Statement::from_string(backend, format!(
                "DELETE FROM auth_user_roles \
                 WHERE user_id = {user_id} AND role_id = (SELECT id FROM auth_roles WHERE name = '{safe_role}')"
            )))
            .await
            .map_err(db_err)?;

        // Audit log
        let admin_id_sql = admin_user_id.map(|id| id.to_string()).unwrap_or_else(|| "NULL".to_string());
        let ip_sql = client_ip.map(|ip| format!("'{}'", ip.replace('\'', "''"))).unwrap_or_else(|| "NULL".to_string());
        self.db
            .execute(sea_orm::Statement::from_string(backend, format!(
                "INSERT INTO auth_audit_log (user_id, action_type, ip_address, details, severity) \
                 VALUES ({admin_id_sql}, 'role_removed', {ip_sql}, \
                 '{{\"user_id\": {user_id}, \"role\": \"{safe_role}\"}}', 'info')",
            )))
            .await
            .map_err(db_err)?;

        Ok(SuccessResponse { success: true, message: None })
    }

    // ================================================================
    // POST /auth-management/v1/users/{user_id}/2fa/setup
    // ================================================================
    pub async fn setup_2fa(
        &self,
        _ctx: &SecurityContext,
        user_id: i64,
    ) -> Result<TwoFactorSetupResponse, DomainError> {
        let backend = self.db.get_database_backend();

        // Get username
        let user_rows = self.db
            .query_all(sea_orm::Statement::from_string(backend, format!(
                "SELECT username FROM auth_users WHERE id = {user_id}"
            )))
            .await
            .map_err(db_err)?;

        let username = user_rows.first()
            .and_then(|r| opt_str(r, 0))
            .ok_or_else(|| DomainError::NotFound(format!("User {user_id} not found")))?;

        // Generate TOTP secret
        let secret = totp_rs::Secret::generate_secret();
        let secret_base32 = secret.to_encoded().to_string();

        let totp = totp_rs::TOTP::new(
            totp_rs::Algorithm::SHA1,
            6,
            1,
            30,
            secret.to_bytes().map_err(|e| DomainError::Database(format!("Secret error: {e}")))?,
            Some("Crico AI Trading Bot".to_string()),
            username.clone(),
        ).map_err(|e| DomainError::Database(format!("TOTP error: {e}")))?;

        // Store secret (not enabled yet)
        self.db
            .execute(sea_orm::Statement::from_string(backend, format!(
                "UPDATE auth_users SET totp_secret = '{}', totp_enabled = FALSE, \
                 updated_at = CURRENT_TIMESTAMP WHERE id = {user_id}",
                secret_base32.replace('\'', "''"),
            )))
            .await
            .map_err(db_err)?;

        // Generate QR code as base64 PNG
        let uri = totp.get_url();
        let qr_code = qrcode::QrCode::new(uri.as_bytes())
            .map_err(|e| DomainError::Database(format!("QR code generation error: {e}")))?;

        let img = qr_code.render::<image::Luma<u8>>().build();
        let mut png_bytes: Vec<u8> = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(&mut png_bytes);
        image::ImageEncoder::write_image(
            encoder,
            img.as_raw(),
            img.width(),
            img.height(),
            image::ExtendedColorType::L8,
        )
        .map_err(|e| DomainError::Database(format!("PNG encoding error: {e}")))?;

        let qr_base64 = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            &png_bytes,
        );

        Ok(TwoFactorSetupResponse {
            success: true,
            secret: secret_base32,
            qr_code: qr_base64,
            uri,
        })
    }

    // ================================================================
    // POST /auth-management/v1/users/{user_id}/2fa/verify
    // ================================================================
    pub async fn verify_2fa(
        &self,
        _ctx: &SecurityContext,
        user_id: i64,
        token: &str,
    ) -> Result<SuccessResponse, DomainError> {
        let backend = self.db.get_database_backend();

        // Get user's TOTP secret
        let user_rows = self.db
            .query_all(sea_orm::Statement::from_string(backend, format!(
                "SELECT username, totp_secret FROM auth_users WHERE id = {user_id}"
            )))
            .await
            .map_err(db_err)?;

        let row = user_rows.first()
            .ok_or_else(|| DomainError::NotFound(format!("User {user_id} not found")))?;

        let username = opt_str(row, 0).unwrap_or_default();
        let secret_str = opt_str(row, 1)
            .ok_or_else(|| DomainError::BadRequest("2FA not set up for this user".to_string()))?;

        let secret = totp_rs::Secret::Encoded(secret_str);
        let totp = totp_rs::TOTP::new(
            totp_rs::Algorithm::SHA1,
            6,
            1,
            30,
            secret.to_bytes().map_err(|e| DomainError::Database(format!("Secret error: {e}")))?,
            Some("Crico AI Trading Bot".to_string()),
            username,
        ).map_err(|e| DomainError::Database(format!("TOTP error: {e}")))?;

        if !totp.check_current(token).unwrap_or(false) {
            return Ok(SuccessResponse { success: false, message: Some("Invalid token".to_string()) });
        }

        // Enable 2FA
        self.db
            .execute(sea_orm::Statement::from_string(backend, format!(
                "UPDATE auth_users SET totp_enabled = TRUE, updated_at = CURRENT_TIMESTAMP WHERE id = {user_id}"
            )))
            .await
            .map_err(db_err)?;

        // Audit log
        self.db
            .execute(sea_orm::Statement::from_string(backend, format!(
                "INSERT INTO auth_audit_log (user_id, action_type, details, severity) \
                 VALUES ({user_id}, '2fa_enabled', '{{\"user_id\": {user_id}}}', 'info')"
            )))
            .await
            .map_err(db_err)?;

        Ok(SuccessResponse { success: true, message: None })
    }

    // ================================================================
    // DELETE /auth-management/v1/users/{user_id}/2fa
    // ================================================================
    pub async fn disable_2fa(
        &self,
        _ctx: &SecurityContext,
        user_id: i64,
        admin_user_id: Option<i64>,
    ) -> Result<SuccessResponse, DomainError> {
        let backend = self.db.get_database_backend();

        self.db
            .execute(sea_orm::Statement::from_string(backend, format!(
                "UPDATE auth_users SET totp_enabled = FALSE, totp_secret = NULL, \
                 updated_at = CURRENT_TIMESTAMP WHERE id = {user_id}"
            )))
            .await
            .map_err(db_err)?;

        let actor_id = admin_user_id.unwrap_or(user_id);
        self.db
            .execute(sea_orm::Statement::from_string(backend, format!(
                "INSERT INTO auth_audit_log (user_id, action_type, details, severity) \
                 VALUES ({actor_id}, '2fa_disabled', '{{\"user_id\": {user_id}}}', 'warning')"
            )))
            .await
            .map_err(db_err)?;

        Ok(SuccessResponse { success: true, message: None })
    }

    // ================================================================
    // GET /auth-management/v1/security/ip-blocks
    // ================================================================
    pub async fn get_ip_blocks(
        &self,
        _ctx: &SecurityContext,
    ) -> Result<IpBlocksResponse, DomainError> {
        let backend = self.db.get_database_backend();

        // Blacklist
        let bl_rows = self.db
            .query_all(sea_orm::Statement::from_string(backend, r#"
                SELECT id, ip_address, reason, is_permanent, blocked_until, auto_blocked,
                       geo_country, geo_city, created_at
                FROM auth_ip_blacklist
                WHERE is_permanent = TRUE OR blocked_until > CURRENT_TIMESTAMP
                ORDER BY created_at DESC
            "#.to_string()))
            .await
            .map_err(db_err)?;

        let blacklist: Vec<BlacklistEntryDto> = bl_rows.iter().map(|row| {
            BlacklistEntryDto {
                id: opt_i64(row, 0),
                ip_address: opt_str(row, 1).unwrap_or_default(),
                reason: opt_str(row, 2),
                is_permanent: opt_bool(row, 3),
                blocked_until: opt_ts(row, 4),
                auto_blocked: opt_bool(row, 5),
                geo_country: opt_str(row, 6),
                geo_city: opt_str(row, 7),
                created_at: opt_ts(row, 8),
            }
        }).collect();

        // Whitelist
        let wl_rows = self.db
            .query_all(sea_orm::Statement::from_string(backend, r#"
                SELECT id, ip_address, description, expires_at, created_at
                FROM auth_ip_whitelist
                WHERE expires_at IS NULL OR expires_at > CURRENT_TIMESTAMP
                ORDER BY created_at DESC
            "#.to_string()))
            .await
            .map_err(db_err)?;

        let whitelist: Vec<WhitelistEntryDto> = wl_rows.iter().map(|row| {
            WhitelistEntryDto {
                id: opt_i64(row, 0),
                ip_address: opt_str(row, 1).unwrap_or_default(),
                description: opt_str(row, 2),
                expires_at: opt_ts(row, 3),
                created_at: opt_ts(row, 4),
            }
        }).collect();

        Ok(IpBlocksResponse { blacklist, whitelist })
    }

    // ================================================================
    // POST /auth-management/v1/security/ip-blocks
    // ================================================================
    pub async fn block_ip(
        &self,
        _ctx: &SecurityContext,
        req: &IpBlockRequest,
        admin_user_id: Option<i64>,
    ) -> Result<IpBlockCreatedResponse, DomainError> {
        let backend = self.db.get_database_backend();

        let blocked_until_sql = req.blocked_until.as_ref()
            .map(|ts| format!("'{}'", ts.replace('\'', "''")))
            .unwrap_or_else(|| "NULL".to_string());
        let admin_id_sql = admin_user_id.map(|id| id.to_string()).unwrap_or_else(|| "NULL".to_string());

        let rows = self.db
            .query_all(sea_orm::Statement::from_string(backend, format!(
                "INSERT INTO auth_ip_blacklist (ip_address, reason, is_permanent, blocked_until, auto_blocked, blocked_by) \
                 VALUES ('{}', '{}', {}, {blocked_until_sql}, FALSE, {admin_id_sql}) RETURNING id",
                req.ip.replace('\'', "''"),
                req.reason.replace('\'', "''"),
                req.is_permanent,
            )))
            .await
            .map_err(db_err)?;

        let block_id = rows.first().map(|r| opt_i64(r, 0)).unwrap_or(0);

        // Audit log
        let ip_sql = format!("'{}'", req.ip.replace('\'', "''"));
        self.db
            .execute(sea_orm::Statement::from_string(backend, format!(
                "INSERT INTO auth_audit_log (user_id, action_type, ip_address, details, severity) \
                 VALUES ({admin_id_sql}, 'ip_blocked', {ip_sql}, \
                 '{{\"reason\": \"{}\", \"is_permanent\": {}}}', 'warning')",
                req.reason.replace('\'', "''").replace('"', "\\\""),
                req.is_permanent,
            )))
            .await
            .map_err(db_err)?;

        Ok(IpBlockCreatedResponse { success: true, block_id })
    }

    // ================================================================
    // DELETE /auth-management/v1/security/ip-blocks/{block_id}
    // ================================================================
    pub async fn unblock_ip(
        &self,
        _ctx: &SecurityContext,
        block_id: i64,
        admin_user_id: Option<i64>,
    ) -> Result<SuccessResponse, DomainError> {
        let backend = self.db.get_database_backend();

        // Get IP before deleting
        let ip_rows = self.db
            .query_all(sea_orm::Statement::from_string(backend, format!(
                "SELECT ip_address FROM auth_ip_blacklist WHERE id = {block_id}"
            )))
            .await
            .map_err(db_err)?;

        let ip_address = ip_rows.first()
            .and_then(|r| opt_str(r, 0))
            .ok_or_else(|| DomainError::NotFound("IP not found in blacklist".to_string()))?;

        self.db
            .execute(sea_orm::Statement::from_string(backend, format!(
                "DELETE FROM auth_ip_blacklist WHERE id = {block_id}"
            )))
            .await
            .map_err(db_err)?;

        // Audit log
        let admin_id_sql = admin_user_id.map(|id| id.to_string()).unwrap_or_else(|| "NULL".to_string());
        self.db
            .execute(sea_orm::Statement::from_string(backend, format!(
                "INSERT INTO auth_audit_log (user_id, action_type, ip_address, details, severity) \
                 VALUES ({admin_id_sql}, 'ip_unblocked', '{}', '{{\"unblocked\": true}}', 'info')",
                ip_address.replace('\'', "''"),
            )))
            .await
            .map_err(db_err)?;

        Ok(SuccessResponse { success: true, message: None })
    }

    // ================================================================
    // POST /auth-management/v1/security/ip-whitelist
    // ================================================================
    pub async fn whitelist_ip(
        &self,
        _ctx: &SecurityContext,
        req: &IpWhitelistRequest,
        admin_user_id: Option<i64>,
    ) -> Result<IpWhitelistCreatedResponse, DomainError> {
        let backend = self.db.get_database_backend();

        let expires_sql = req.expires_at.as_ref()
            .map(|ts| format!("'{}'", ts.replace('\'', "''")))
            .unwrap_or_else(|| "NULL".to_string());
        let admin_id_sql = admin_user_id.map(|id| id.to_string()).unwrap_or_else(|| "NULL".to_string());

        let rows = self.db
            .query_all(sea_orm::Statement::from_string(backend, format!(
                "INSERT INTO auth_ip_whitelist (ip_address, description, added_by, expires_at) \
                 VALUES ('{}', '{}', {admin_id_sql}, {expires_sql}) RETURNING id",
                req.ip.replace('\'', "''"),
                req.description.replace('\'', "''"),
            )))
            .await
            .map_err(db_err)?;

        let whitelist_id = rows.first().map(|r| opt_i64(r, 0)).unwrap_or(0);

        // Audit log
        self.db
            .execute(sea_orm::Statement::from_string(backend, format!(
                "INSERT INTO auth_audit_log (user_id, action_type, ip_address, details, severity) \
                 VALUES ({admin_id_sql}, 'ip_whitelisted', '{}', \
                 '{{\"description\": \"{}\"}}', 'info')",
                req.ip.replace('\'', "''"),
                req.description.replace('\'', "''").replace('"', "\\\""),
            )))
            .await
            .map_err(db_err)?;

        Ok(IpWhitelistCreatedResponse { success: true, whitelist_id })
    }

    // ================================================================
    // DELETE /auth-management/v1/security/ip-whitelist/{whitelist_id}
    // ================================================================
    pub async fn remove_whitelist(
        &self,
        _ctx: &SecurityContext,
        whitelist_id: i64,
        admin_user_id: Option<i64>,
    ) -> Result<SuccessResponse, DomainError> {
        let backend = self.db.get_database_backend();

        // Get IP before deleting
        let ip_rows = self.db
            .query_all(sea_orm::Statement::from_string(backend, format!(
                "SELECT ip_address FROM auth_ip_whitelist WHERE id = {whitelist_id}"
            )))
            .await
            .map_err(db_err)?;

        let ip_address = ip_rows.first()
            .and_then(|r| opt_str(r, 0))
            .ok_or_else(|| DomainError::NotFound("IP not found in whitelist".to_string()))?;

        self.db
            .execute(sea_orm::Statement::from_string(backend, format!(
                "DELETE FROM auth_ip_whitelist WHERE id = {whitelist_id}"
            )))
            .await
            .map_err(db_err)?;

        // Audit log
        let admin_id_sql = admin_user_id.map(|id| id.to_string()).unwrap_or_else(|| "NULL".to_string());
        self.db
            .execute(sea_orm::Statement::from_string(backend, format!(
                "INSERT INTO auth_audit_log (user_id, action_type, ip_address, details, severity) \
                 VALUES ({admin_id_sql}, 'ip_whitelist_removed', '{}', '{{\"removed\": true}}', 'info')",
                ip_address.replace('\'', "''"),
            )))
            .await
            .map_err(db_err)?;

        Ok(SuccessResponse { success: true, message: None })
    }

    // ================================================================
    // GET /auth-management/v1/audit-logs
    // ================================================================
    pub async fn get_audit_logs(
        &self,
        _ctx: &SecurityContext,
        limit: i64,
        offset: i64,
        user_id_filter: Option<i64>,
        action_type: Option<&str>,
        start_date: Option<&str>,
        end_date: Option<&str>,
    ) -> Result<AuditLogsResponse, DomainError> {
        let backend = self.db.get_database_backend();

        let mut where_parts: Vec<String> = Vec::new();

        if let Some(uid) = user_id_filter {
            where_parts.push(format!("al.user_id = {uid}"));
        }
        if let Some(at) = action_type {
            where_parts.push(format!("al.action_type = '{}'", at.replace('\'', "''")));
        }
        if let Some(sd) = start_date {
            where_parts.push(format!("al.created_at >= '{}'", sd.replace('\'', "''")));
        }
        if let Some(ed) = end_date {
            where_parts.push(format!("al.created_at <= '{}'", ed.replace('\'', "''")));
        }

        let where_clause = if where_parts.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_parts.join(" AND "))
        };

        let query = format!(
            r#"SELECT al.id, al.user_id, u.username, al.action_type, al.ip_address,
                      al.details, al.severity, al.created_at
               FROM auth_audit_log al
               LEFT JOIN auth_users u ON u.id = al.user_id
               {where_clause}
               ORDER BY al.created_at DESC
               LIMIT {limit} OFFSET {offset}"#
        );

        let rows = self.db
            .query_all(sea_orm::Statement::from_string(backend, query))
            .await
            .map_err(db_err)?;

        let logs: Vec<AuditLogEntryDto> = rows.iter().map(|row| {
            AuditLogEntryDto {
                id: opt_i64(row, 0),
                user_id: opt_i64_nullable(row, 1),
                username: opt_str(row, 2),
                action_type: opt_str(row, 3).unwrap_or_default(),
                ip_address: opt_str(row, 4),
                details: opt_str(row, 5),
                severity: opt_str(row, 6),
                created_at: opt_ts(row, 7),
            }
        }).collect();

        Ok(AuditLogsResponse { logs })
    }

    // ================================================================
    // GET /auth-management/v1/sessions (placeholder)
    // ================================================================
    pub async fn get_sessions(
        &self,
        _ctx: &SecurityContext,
    ) -> Result<SessionsResponse, DomainError> {
        Ok(SessionsResponse {
            sessions: vec![],
            message: "Session management via auth-proxy".to_string(),
        })
    }

    // ================================================================
    // DELETE /auth-management/v1/sessions/{session_id} (placeholder)
    // ================================================================
    pub async fn revoke_session(
        &self,
        _ctx: &SecurityContext,
        _session_id: i64,
    ) -> Result<SuccessResponse, DomainError> {
        Ok(SuccessResponse {
            success: true,
            message: Some("Session revocation via auth-proxy".to_string()),
        })
    }
}
