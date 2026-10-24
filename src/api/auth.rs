use actix_web::cookie::{Cookie, SameSite};
use actix_web::dev::Payload;
use actix_web::{web, FromRequest, HttpRequest, HttpResponse, Responder};
use futures_util::future::{FutureExt, LocalBoxFuture};
use serde_json::json;

use crate::auth;
use crate::db::models::{
    AddAllowlistRequest, AdminUser, GoogleCallbackQuery, LoginRequest, LoginResponse,
};
use crate::db::organizations::get_default_organization;
use crate::db::DbPool;

fn url_encode(input: &str) -> String {
    let mut encoded = String::new();
    for byte in input.bytes() {
        match byte {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            _ => {
                encoded.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    encoded
}

// ==============================================================================
// Token Extraction & Authenticated Admin Extractor
// ==============================================================================

pub fn extract_token(req: &HttpRequest) -> Option<String> {
    if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                return Some(auth_str[7..].trim().to_string());
            }
        }
    }
    // Check HTTP-only cookie
    if let Some(cookie) = req.cookie("admin_token") {
        let val = cookie.value().trim();
        if !val.is_empty() {
            return Some(val.to_string());
        }
    }
    None
}

/// Strongly typed extractor that guarantees the request is from an authenticated admin
pub struct AuthenticatedAdmin(pub AdminUser);

impl FromRequest for AuthenticatedAdmin {
    type Error = actix_web::Error;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let req = req.clone();
        async move {
            let pool = req
                .app_data::<web::Data<DbPool>>()
                .ok_or_else(|| actix_web::error::ErrorInternalServerError("Database pool unavailable"))?;

            let token = extract_token(&req).ok_or_else(|| {
                actix_web::error::ErrorUnauthorized(json!({
                    "error": "Authentication required. Please sign in with an authorized Google account."
                }))
            })?;

            let admin = auth::validate_session(pool.get_ref(), &token)
                .await
                .map_err(|e| {
                    log::error!("Database error during session validation: {}", e);
                    actix_web::error::ErrorInternalServerError("Database error")
                })?
                .ok_or_else(|| {
                    actix_web::error::ErrorUnauthorized(json!({
                        "error": "Session expired or invalid. Please sign in again."
                    }))
                })?;

            Ok(AuthenticatedAdmin(admin))
        }
        .boxed_local()
    }
}

// ==============================================================================
// Google OAuth 2.0 / OpenID Connect Endpoints
// ==============================================================================

fn get_redirect_uri(req: &HttpRequest) -> String {
    if let Ok(uri) = std::env::var("GOOGLE_REDIRECT_URI") {
        if !uri.trim().is_empty() {
            return uri.trim().to_string();
        }
    }

    let connection_info = req.connection_info();
    let scheme = connection_info.scheme();
    let host = connection_info.host();
    format!("{}://{}/api/auth/google/callback", scheme, host)
}

/// GET /api/auth/google/url
/// Returns Google OAuth 2.0 authorization URL with CSRF state and nonce
pub async fn google_auth_url(pool: web::Data<DbPool>, req: HttpRequest) -> impl Responder {
    let client_id = match std::env::var("GOOGLE_CLIENT_ID") {
        Ok(id) if !id.trim().is_empty() => id.trim().to_string(),
        _ => {
            return HttpResponse::Ok().json(json!({
                "configured": false,
                "message": "Google OAuth is not configured. Please set GOOGLE_CLIENT_ID and GOOGLE_CLIENT_SECRET."
            }));
        }
    };

    let redirect_uri = get_redirect_uri(&req);

    // Create state & nonce with 10-minute expiration
    let (state, nonce) = match auth::create_oauth_state(pool.get_ref(), Some(&redirect_uri)).await {
        Ok(s) => s,
        Err(e) => {
            log::error!("Failed to create oauth state: {}", e);
            return HttpResponse::InternalServerError().json(json!({
                "error": "Failed to initialize OAuth state"
            }));
        }
    };

    // Google OpenID Connect authorization URL
    let auth_url = format!(
        "https://accounts.google.com/o/oauth2/v2/auth?\
        client_id={}&\
        redirect_uri={}&\
        response_type=code&\
        scope=openid%20email%20profile&\
        state={}&\
        nonce={}&\
        prompt=select_account",
        url_encode(&client_id),
        url_encode(&redirect_uri),
        url_encode(&state),
        url_encode(&nonce),
    );

    // Set double-submit CSRF cookie for extra security
    let state_cookie = Cookie::build("oauth_state", &state)
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(actix_web::cookie::time::Duration::minutes(10))
        .finish();

    HttpResponse::Ok()
        .cookie(state_cookie)
        .json(json!({
            "configured": true,
            "url": auth_url,
            "state": state
        }))
}

/// GET /api/auth/google/callback
/// Handles Google OAuth 2.0 redirect, state verification, code exchange, allowlist checking
pub async fn google_callback(
    pool: web::Data<DbPool>,
    req: HttpRequest,
    query: web::Query<GoogleCallbackQuery>,
) -> impl Responder {
    if let Some(err) = &query.error {
        log::warn!("Google OAuth reported error: {}", err);
        return HttpResponse::Found()
            .append_header(("Location", "/admin?error=google_cancelled"))
            .finish();
    }

    let code = match &query.code {
        Some(c) if !c.trim().is_empty() => c.trim(),
        _ => {
            return HttpResponse::Found()
                .append_header(("Location", "/admin?error=missing_code"))
                .finish();
        }
    };

    let state = match &query.state {
        Some(s) if !s.trim().is_empty() => s.trim(),
        _ => {
            return HttpResponse::Found()
                .append_header(("Location", "/admin?error=missing_state"))
                .finish();
        }
    };

    // 1. Verify and consume OAuth state (one-time use, prevents replay attacks)
    let state_valid = match auth::verify_and_consume_oauth_state(pool.get_ref(), state).await {
        Ok(res) => res.is_some(),
        Err(e) => {
            log::error!("Database error verifying oauth state: {}", e);
            false
        }
    };

    // Also check double-submit cookie if present
    if let Some(cookie_state) = req.cookie("oauth_state") {
        if cookie_state.value() != state {
            log::warn!("OAuth state cookie mismatch: cookie vs query param");
            return HttpResponse::Found()
                .append_header(("Location", "/admin?error=invalid_state"))
                .finish();
        }
    }

    if !state_valid {
        log::warn!("Invalid or expired OAuth state token rejected: {}", state);
        return HttpResponse::Found()
            .append_header(("Location", "/admin?error=invalid_state"))
            .finish();
    }

    // 2. Fetch Google Client credentials
    let client_id = match std::env::var("GOOGLE_CLIENT_ID") {
        Ok(id) if !id.trim().is_empty() => id.trim().to_string(),
        _ => {
            return HttpResponse::Found()
                .append_header(("Location", "/admin?error=oauth_unconfigured"))
                .finish();
        }
    };

    let client_secret = match std::env::var("GOOGLE_CLIENT_SECRET") {
        Ok(sec) if !sec.trim().is_empty() => sec.trim().to_string(),
        _ => {
            return HttpResponse::Found()
                .append_header(("Location", "/admin?error=oauth_unconfigured"))
                .finish();
        }
    };

    let redirect_uri = get_redirect_uri(&req);

    // 3. Exchange authorization code with Google token endpoint
    let token_res = match auth::exchange_google_code(&client_id, &client_secret, &redirect_uri, code).await {
        Ok(t) => t,
        Err(e) => {
            log::error!("Failed to exchange Google OAuth code: {}", e);
            return HttpResponse::Found()
                .append_header(("Location", "/admin?error=token_exchange_failed"))
                .finish();
        }
    };

    // 4. Fetch user identity profile from OpenID Connect endpoint
    let user_info = match auth::fetch_google_userinfo(&token_res.access_token).await {
        Ok(u) => u,
        Err(e) => {
            log::error!("Failed to fetch Google userinfo: {}", e);
            return HttpResponse::Found()
                .append_header(("Location", "/admin?error=userinfo_failed"))
                .finish();
        }
    };

    if user_info.email_verified != Some(true) {
        log::warn!("Google account email is not verified: {}", user_info.email);
        return HttpResponse::Found()
            .append_header(("Location", "/admin?error=email_not_verified"))
            .finish();
    }

    // 5. SERVER-SIDE EMAIL ALLOWLIST VALIDATION
    let allowed_role = match auth::is_email_allowed(pool.get_ref(), &user_info.email).await {
        Ok(role) => role,
        Err(e) => {
            log::error!("Error checking email allowlist: {}", e);
            return HttpResponse::Found()
                .append_header(("Location", "/admin?error=database_error"))
                .finish();
        }
    };

    let role = match allowed_role {
        Some(r) => r,
        None => {
            log::warn!(
                "SECURITY ALERT: Rejected unauthorized Google sign-in attempt from unlisted account: email='{}', sub='{}'",
                user_info.email,
                user_info.sub
            );
            let encoded_email = url_encode(&user_info.email);
            return HttpResponse::Found()
                .append_header((
                    "Location",
                    format!("/admin?error=unauthorized_email&email={}", encoded_email),
                ))
                .finish();
        }
    };

    // 6. Upsert Admin Profile in PostgreSQL
    let admin = match auth::upsert_google_admin(pool.get_ref(), &user_info, &role).await {
        Ok(a) => a,
        Err(e) => {
            log::error!("Failed to save admin user in database: {}", e);
            return HttpResponse::Found()
                .append_header(("Location", "/admin?error=database_error"))
                .finish();
        }
    };

    // 7. Create secure server-side session
    let user_agent = req.headers().get("User-Agent").and_then(|h| h.to_str().ok());
    let ip_address = req.connection_info().realip_remote_addr().map(|s| s.to_string());

    let token = match auth::create_session_with_meta(pool.get_ref(), &admin.id, user_agent, ip_address.as_deref()).await {
        Ok(t) => t,
        Err(e) => {
            log::error!("Failed to create admin session: {}", e);
            return HttpResponse::Found()
                .append_header(("Location", "/admin?error=session_error"))
                .finish();
        }
    };

    // 8. Set HTTP-only, secure session cookie
    let session_cookie = Cookie::build("admin_token", &token)
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(actix_web::cookie::time::Duration::days(7))
        .finish();

    // Clear temporary oauth_state cookie
    let clear_state_cookie = Cookie::build("oauth_state", "")
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(actix_web::cookie::time::Duration::seconds(0))
        .finish();

    log::info!(
        "Administrator authenticated successfully via Google: email='{}', role='{}'",
        admin.email.as_deref().unwrap_or(""),
        admin.role
    );

    HttpResponse::Found()
        .cookie(session_cookie)
        .cookie(clear_state_cookie)
        .append_header(("Location", "/admin?login_success=1"))
        .finish()
}

// ==============================================================================
// Admin Allowlist Management Endpoints (Protected)
// ==============================================================================

/// GET /api/admin/allowlist
pub async fn list_allowlist(
    pool: web::Data<DbPool>,
    _admin: AuthenticatedAdmin,
) -> impl Responder {
    match auth::get_allowlist(pool.get_ref()).await {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(e) => {
            log::error!("Failed to list admin allowlist: {}", e);
            HttpResponse::InternalServerError().json(json!({ "error": "Database error" }))
        }
    }
}

/// POST /api/admin/allowlist
pub async fn add_allowlist_entry(
    pool: web::Data<DbPool>,
    admin: AuthenticatedAdmin,
    body: web::Json<AddAllowlistRequest>,
) -> impl Responder {
    if admin.0.role != "superadmin" {
        return HttpResponse::Forbidden().json(json!({
            "error": "Only superadmin accounts can manage the administrator allowlist"
        }));
    }

    let role = body.role.as_deref().unwrap_or("admin");
    match auth::add_to_allowlist(pool.get_ref(), &body.email, role, body.notes.as_deref()).await {
        Ok(entry) => HttpResponse::Created().json(entry),
        Err(e) => {
            log::error!("Failed to add to allowlist: {}", e);
            HttpResponse::InternalServerError().json(json!({ "error": "Database error" }))
        }
    }
}

/// DELETE /api/admin/allowlist/{email}
pub async fn remove_allowlist_entry(
    pool: web::Data<DbPool>,
    admin: AuthenticatedAdmin,
    path: web::Path<String>,
) -> impl Responder {
    if admin.0.role != "superadmin" {
        return HttpResponse::Forbidden().json(json!({
            "error": "Only superadmin accounts can manage the administrator allowlist"
        }));
    }

    let email = path.into_inner();
    match auth::remove_from_allowlist(pool.get_ref(), &email).await {
        Ok(true) => HttpResponse::Ok().json(json!({ "status": "deleted" })),
        Ok(false) => HttpResponse::NotFound().json(json!({ "error": "Email not found on allowlist" })),
        Err(e) => {
            log::error!("Failed to delete from allowlist: {}", e);
            HttpResponse::InternalServerError().json(json!({ "error": "Database error" }))
        }
    }
}

// ==============================================================================
// Existing / Legacy Endpoints & Session Management
// ==============================================================================

pub async fn login(pool: web::Data<DbPool>, req: HttpRequest, body: web::Json<LoginRequest>) -> impl Responder {
    let admin_row = sqlx::query(
        r#"
        SELECT id, organization_id, username, email, name, avatar_url, role, password_hash, salt
        FROM admins
        WHERE username = $1 OR (email IS NOT NULL AND LOWER(email) = LOWER($1))
        "#,
    )
    .bind(&body.username)
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
    let email: Option<String> = row.try_get("email").ok();
    let name: Option<String> = row.try_get("name").ok();
    let avatar_url: Option<String> = row.try_get("avatar_url").ok();
    let role: String = row.try_get("role").unwrap_or_else(|_| "admin".to_string());
    let password_hash: Option<String> = row.try_get("password_hash").ok();
    let salt: Option<String> = row.try_get("salt").ok();

    if let (Some(ph), Some(s)) = (password_hash, salt) {
        if !auth::verify_password(&body.password, &s, &ph) {
            return HttpResponse::Unauthorized().json(json!({
                "error": "Invalid username or password"
            }));
        }
    } else {
        return HttpResponse::Unauthorized().json(json!({
            "error": "This account must sign in using Google Single Sign-On"
        }));
    }

    let user_agent = req.headers().get("User-Agent").and_then(|h| h.to_str().ok());
    let ip_address = req.connection_info().realip_remote_addr().map(|s| s.to_string());

    let token = match auth::create_session_with_meta(pool.get_ref(), &admin_id, user_agent, ip_address.as_deref()).await {
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
        email,
        name,
        avatar_url,
        role,
    };

    let session_cookie = Cookie::build("admin_token", &token)
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(actix_web::cookie::time::Duration::days(7))
        .finish();

    HttpResponse::Ok()
        .cookie(session_cookie)
        .json(LoginResponse {
            token,
            admin,
            organization: org,
        })
}

pub async fn logout(pool: web::Data<DbPool>, req: HttpRequest) -> impl Responder {
    if let Some(token) = extract_token(&req) {
        let _ = auth::delete_session(pool.get_ref(), &token).await;
    }

    // Clear HTTP-only session cookie
    let clear_cookie = Cookie::build("admin_token", "")
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(actix_web::cookie::time::Duration::seconds(0))
        .finish();

    HttpResponse::Ok()
        .cookie(clear_cookie)
        .json(json!({ "status": "logged_out" }))
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
