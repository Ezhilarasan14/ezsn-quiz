use actix_web::{web, HttpRequest, HttpResponse, Responder};
use serde_json::json;

use crate::auth;
use crate::db::models::{AdminUser, LoginRequest, LoginResponse};
use crate::db::organizations::get_default_organization;
use crate::db::DbPool;

pub fn extract_token(req: &HttpRequest) -> Option<String> {
    if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                return Some(auth_str[7..].trim().to_string());
            }
        }
    }
    // Check cookie
    if let Some(cookie) = req.cookie("admin_token") {
        return Some(cookie.value().to_string());
    }
    None
}

pub async fn login(pool: web::Data<DbPool>, req: web::Json<LoginRequest>) -> impl Responder {
    let admin_row = sqlx::query(
        r#"
        SELECT id, organization_id, username, password_hash, salt
        FROM admins
        WHERE username = ?
        "#,
    )
    .bind(&req.username)
    .fetch_optional(pool.get_ref())
    .await;

    let row = match admin_row {
        Ok(Some(r)) => r,
        Ok(None) => {
            return HttpResponse::Unauthorized().json(json!({
                "error": "Invalid username or password"
            }));
        }
        Err(e) => {
            log::error!("Database error during login: {}", e);
            return HttpResponse::InternalServerError().json(json!({
                "error": "Database error"
            }));
        }
    };

    use sqlx::Row;
    let admin_id: String = row.get("id");
    let organization_id: Option<String> = row.get("organization_id");
    let username: String = row.get("username");
    let password_hash: String = row.get("password_hash");
    let salt: String = row.get("salt");

    if !auth::verify_password(&req.password, &salt, &password_hash) {
        return HttpResponse::Unauthorized().json(json!({
            "error": "Invalid username or password"
        }));
    }

    let token = match auth::create_session(pool.get_ref(), &admin_id).await {
        Ok(t) => t,
        Err(e) => {
            log::error!("Failed to create admin session: {}", e);
            return HttpResponse::InternalServerError().json(json!({
                "error": "Failed to create session"
            }));
        }
    };

    let org = get_default_organization(pool.get_ref()).await.ok();

    let admin = AdminUser {
        id: admin_id,
        organization_id,
        username,
    };

    HttpResponse::Ok().json(LoginResponse {
        token,
        admin,
        organization: org,
    })
}

pub async fn logout(pool: web::Data<DbPool>, req: HttpRequest) -> impl Responder {
    if let Some(token) = extract_token(&req) {
        let _ = auth::delete_session(pool.get_ref(), &token).await;
    }
    HttpResponse::Ok().json(json!({ "status": "logged_out" }))
}

pub async fn me(pool: web::Data<DbPool>, req: HttpRequest) -> impl Responder {
    let token = match extract_token(&req) {
        Some(t) => t,
        None => {
            return HttpResponse::Unauthorized().json(json!({
                "error": "Authentication token missing"
            }));
        }
    };

    match auth::validate_session(pool.get_ref(), &token).await {
        Ok(Some(admin)) => {
            let org = get_default_organization(pool.get_ref()).await.ok();
            HttpResponse::Ok().json(json!({
                "admin": admin,
                "organization": org
            }))
        }
        Ok(None) => HttpResponse::Unauthorized().json(json!({
            "error": "Session expired or invalid"
        })),
        Err(e) => {
            log::error!("Error validating admin session: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "error": "Database error"
            }))
        }
    }
}
