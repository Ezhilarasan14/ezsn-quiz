use crate::db::models::{AdminAllowlistEntry, AdminUser, GoogleTokenResponse, GoogleUserInfo};
use crate::db::DbPool;
use rand::RngCore;
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub fn hash_password(password: &str) -> (String, String) {
    let mut salt_bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut salt_bytes);
    let salt = hex::encode(salt_bytes);

    let mut hasher = Sha256::new();
    hasher.update(salt.as_bytes());
    hasher.update(password.as_bytes());
    let hash = hex::encode(hasher.finalize());

    (salt, hash)
}

pub fn verify_password(password: &str, salt: &str, expected_hash: &str) -> bool {
    let mut hasher = Sha256::new();
    hasher.update(salt.as_bytes());
    hasher.update(password.as_bytes());
    let calculated_hash = hex::encode(hasher.finalize());

    calculated_hash == expected_hash
}

pub async fn create_session(pool: &DbPool, admin_id: &str) -> Result<String, sqlx::Error> {
    create_session_with_meta(pool, admin_id, None, None).await
}

pub async fn create_session_with_meta(
    pool: &DbPool,
    admin_id: &str,
    user_agent: Option<&str>,
    ip_address: Option<&str>,
) -> Result<String, sqlx::Error> {
    let mut token_bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut token_bytes);
    let token = hex::encode(token_bytes);

    // 7 days expiration
    let expires_at = chrono::Utc::now().naive_utc() + chrono::Duration::days(7);

    sqlx::query(
        r#"
        INSERT INTO admin_sessions (token, admin_id, expires_at, created_at, user_agent, ip_address, last_active_at)
        VALUES ($1, $2, $3, CURRENT_TIMESTAMP, $4, $5, CURRENT_TIMESTAMP)
        "#,
    )
    .bind(&token)
    .bind(admin_id)
    .bind(expires_at)
    .bind(user_agent)
    .bind(ip_address)
    .execute(pool)
    .await?;

    Ok(token)
}

pub async fn validate_session(pool: &DbPool, token: &str) -> Result<Option<AdminUser>, sqlx::Error> {
    let admin = sqlx::query_as::<_, AdminUser>(
        r#"
        SELECT a.id, a.organization_id, a.username, a.email, a.name, a.avatar_url, a.role
        FROM admin_sessions s
        JOIN admins a ON s.admin_id = a.id
        WHERE s.token = $1 AND s.expires_at > CURRENT_TIMESTAMP
        "#,
    )
    .bind(token)
    .fetch_optional(pool)
    .await?;

    if admin.is_some() {
        // Touch last_active_at asynchronously
        let _ = sqlx::query(
            "UPDATE admin_sessions SET last_active_at = CURRENT_TIMESTAMP WHERE token = $1",
        )
        .bind(token)
        .execute(pool)
        .await;
    }

    Ok(admin)
}

pub async fn delete_session(pool: &DbPool, token: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM admin_sessions WHERE token = $1")
        .bind(token)
        .execute(pool)
        .await?;
    Ok(())
}

// ==============================================================================
// Google OAuth 2.0 / OIDC & State Management
// ==============================================================================

pub async fn create_oauth_state(
    pool: &DbPool,
    redirect_url: Option<&str>,
) -> Result<(String, String), sqlx::Error> {
    let mut state_bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut state_bytes);
    let state = hex::encode(state_bytes);

    let mut nonce_bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = hex::encode(nonce_bytes);

    // 10 minutes state expiration
    let expires_at = chrono::Utc::now().naive_utc() + chrono::Duration::minutes(10);

    // Clean up expired states first
    let _ = sqlx::query("DELETE FROM oauth_states WHERE expires_at < CURRENT_TIMESTAMP")
        .execute(pool)
        .await;

    sqlx::query(
        r#"
        INSERT INTO oauth_states (state, nonce, redirect_url, expires_at, created_at)
        VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP)
        "#,
    )
    .bind(&state)
    .bind(&nonce)
    .bind(redirect_url)
    .bind(expires_at)
    .execute(pool)
    .await?;

    Ok((state, nonce))
}

pub async fn verify_and_consume_oauth_state(
    pool: &DbPool,
    state: &str,
) -> Result<Option<String>, sqlx::Error> {
    let row = sqlx::query_scalar::<_, Option<String>>(
        r#"
        DELETE FROM oauth_states
        WHERE state = $1 AND expires_at > CURRENT_TIMESTAMP
        RETURNING redirect_url
        "#,
    )
    .bind(state)
    .fetch_optional(pool)
    .await?;

    Ok(row.unwrap_or(None))
}

// Exchange authorization code with Google token endpoint
pub async fn exchange_google_code(
    client_id: &str,
    client_secret: &str,
    redirect_uri: &str,
    code: &str,
) -> Result<GoogleTokenResponse, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

    let params = [
        ("code", code),
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("redirect_uri", redirect_uri),
        ("grant_type", "authorization_code"),
    ];

    let res = client
        .post("https://oauth2.googleapis.com/token")
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Failed to contact Google OAuth token endpoint: {}", e))?;

    if !res.status().is_success() {
        let err_body = res.text().await.unwrap_or_default();
        return Err(format!("Google token exchange failed: {}", err_body));
    }

    res.json::<GoogleTokenResponse>()
        .await
        .map_err(|e| format!("Failed to parse Google token response: {}", e))
}

// Retrieve user info from Google OpenID Connect endpoint
pub async fn fetch_google_userinfo(access_token: &str) -> Result<GoogleUserInfo, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

    let res = client
        .get("https://openidconnect.googleapis.com/v1/userinfo")
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| format!("Failed to contact Google userinfo endpoint: {}", e))?;

    if !res.status().is_success() {
        let err_body = res.text().await.unwrap_or_default();
        return Err(format!("Google userinfo request failed: {}", err_body));
    }

    res.json::<GoogleUserInfo>()
        .await
        .map_err(|e| format!("Failed to parse Google userinfo: {}", e))
}

// ==============================================================================
// Server-side Email Allowlist & Authorization
// ==============================================================================

pub async fn is_email_allowed(pool: &DbPool, email: &str) -> Result<Option<String>, sqlx::Error> {
    let clean_email = email.trim().to_lowercase();

    // 1. Check environment variable allowlist: ADMIN_ALLOWED_EMAILS="admin@ezsnapps.in,ezhil@tocode.ai"
    if let Ok(allowed_env) = std::env::var("ADMIN_ALLOWED_EMAILS") {
        for env_email in allowed_env.split(',') {
            if env_email.trim().to_lowercase() == clean_email {
                return Ok(Some("superadmin".to_string()));
            }
        }
    }

    // 2. Check PostgreSQL database allowlist table
    let role = sqlx::query_scalar::<_, String>(
        r#"
        SELECT role
        FROM admin_allowlist
        WHERE LOWER(email) = $1
        "#,
    )
    .bind(&clean_email)
    .fetch_optional(pool)
    .await?;

    Ok(role)
}

pub async fn upsert_google_admin(
    pool: &DbPool,
    user_info: &GoogleUserInfo,
    role: &str,
) -> Result<AdminUser, sqlx::Error> {
    let clean_email = user_info.email.trim().to_lowercase();
    let display_name = user_info
        .name
        .clone()
        .unwrap_or_else(|| clean_email.split('@').next().unwrap_or("Admin").to_string());
    let username = clean_email.split('@').next().unwrap_or("admin").to_string();
    let admin_id = Uuid::new_v4().to_string();

    let admin = sqlx::query_as::<_, AdminUser>(
        r#"
        INSERT INTO admins (id, organization_id, username, email, google_sub, name, avatar_url, role, created_at, updated_at)
        VALUES ($1, 'org_default', $2, $3, $4, $5, $6, $7, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
        ON CONFLICT (email) DO UPDATE SET
            google_sub = EXCLUDED.google_sub,
            name = EXCLUDED.name,
            avatar_url = EXCLUDED.avatar_url,
            role = EXCLUDED.role,
            updated_at = CURRENT_TIMESTAMP
        RETURNING id, organization_id, username, email, name, avatar_url, role
        "#,
    )
    .bind(&admin_id)
    .bind(&username)
    .bind(&clean_email)
    .bind(&user_info.sub)
    .bind(&display_name)
    .bind(&user_info.picture)
    .bind(role)
    .fetch_one(pool)
    .await?;

    Ok(admin)
}

// Allowlist management
pub async fn get_allowlist(pool: &DbPool) -> Result<Vec<AdminAllowlistEntry>, sqlx::Error> {
    sqlx::query_as::<_, AdminAllowlistEntry>(
        "SELECT email, role, notes, created_at, updated_at FROM admin_allowlist ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await
}

pub async fn add_to_allowlist(
    pool: &DbPool,
    email: &str,
    role: &str,
    notes: Option<&str>,
) -> Result<AdminAllowlistEntry, sqlx::Error> {
    let clean_email = email.trim().to_lowercase();
    sqlx::query_as::<_, AdminAllowlistEntry>(
        r#"
        INSERT INTO admin_allowlist (email, role, notes, created_at, updated_at)
        VALUES ($1, $2, $3, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
        ON CONFLICT (email) DO UPDATE SET
            role = EXCLUDED.role,
            notes = EXCLUDED.notes,
            updated_at = CURRENT_TIMESTAMP
        RETURNING email, role, notes, created_at, updated_at
        "#,
    )
    .bind(&clean_email)
    .bind(role)
    .bind(notes)
    .fetch_one(pool)
    .await
}

pub async fn remove_from_allowlist(pool: &DbPool, email: &str) -> Result<bool, sqlx::Error> {
    let clean_email = email.trim().to_lowercase();
    let res = sqlx::query("DELETE FROM admin_allowlist WHERE LOWER(email) = $1")
        .bind(&clean_email)
        .execute(pool)
        .await?;
    Ok(res.rows_affected() > 0)
}

// Default admin initialization
pub async fn ensure_default_admin(pool: &DbPool) -> Result<(), sqlx::Error> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM admins")
        .fetch_one(pool)
        .await?;

    if count == 0 {
        let admin_id = Uuid::new_v4().to_string();
        let (salt, password_hash) = hash_password("admin123");

        sqlx::query(
            r#"
            INSERT INTO admins (id, organization_id, username, email, role, password_hash, salt, created_at, updated_at)
            VALUES ($1, 'org_default', 'admin', 'admin@ezsnapps.in', 'superadmin', $2, $3, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
            "#,
        )
        .bind(&admin_id)
        .bind(&password_hash)
        .bind(&salt)
        .execute(pool)
        .await?;

        log::info!("Initialized default administrator: username='admin', password='admin123', email='admin@ezsnapps.in'");
    }

    // Seed allowlist from environment variable if present
    if let Ok(allowed_env) = std::env::var("ADMIN_ALLOWED_EMAILS") {
        for email in allowed_env.split(',') {
            let clean = email.trim().to_lowercase();
            if !clean.is_empty() {
                let _ = sqlx::query(
                    r#"
                    INSERT INTO admin_allowlist (email, role, notes, created_at, updated_at)
                    VALUES ($1, 'superadmin', 'Configured via ADMIN_ALLOWED_EMAILS', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
                    ON CONFLICT (email) DO NOTHING
                    "#,
                )
                .bind(&clean)
                .execute(pool)
                .await;
            }
        }
    }

    Ok(())
}
