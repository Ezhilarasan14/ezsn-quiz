use crate::db::models::AdminUser;
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
    let mut token_bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut token_bytes);
    let token = hex::encode(token_bytes);

    // 7 days expiration
    let expires_at = chrono::Utc::now().naive_utc() + chrono::Duration::days(7);

    sqlx::query(
        r#"
        INSERT INTO admin_sessions (token, admin_id, expires_at, created_at)
        VALUES ($1, $2, $3, CURRENT_TIMESTAMP)
        "#,
    )
    .bind(&token)
    .bind(admin_id)
    .bind(expires_at)
    .execute(pool)
    .await?;

    Ok(token)
}

pub async fn validate_session(pool: &DbPool, token: &str) -> Result<Option<AdminUser>, sqlx::Error> {
    let admin = sqlx::query_as::<_, AdminUser>(
        r#"
        SELECT a.id, a.organization_id, a.username
        FROM admin_sessions s
        JOIN admins a ON s.admin_id = a.id
        WHERE s.token = $1 AND s.expires_at > CURRENT_TIMESTAMP
        "#,
    )
    .bind(token)
    .fetch_optional(pool)
    .await?;

    Ok(admin)
}

pub async fn delete_session(pool: &DbPool, token: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM admin_sessions WHERE token = $1")
        .bind(token)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn ensure_default_admin(pool: &DbPool) -> Result<(), sqlx::Error> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM admins")
        .fetch_one(pool)
        .await?;

    if count == 0 {
        let admin_id = Uuid::new_v4().to_string();
        let (salt, password_hash) = hash_password("admin123");

        sqlx::query(
            r#"
            INSERT INTO admins (id, organization_id, username, password_hash, salt, created_at, updated_at)
            VALUES ($1, 'org_default', 'admin', $2, $3, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
            "#,
        )
        .bind(&admin_id)
        .bind(&password_hash)
        .bind(&salt)
        .execute(pool)
        .await?;

        log::info!("Initialized default administrator: username='admin', password='admin123'");
    }

    Ok(())
}
