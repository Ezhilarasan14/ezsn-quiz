use ezsn_quiz::auth;
use ezsn_quiz::db::{self, DbPool};
use ezsn_quiz::db::models::GoogleUserInfo;

#[tokio::test]
async fn test_google_oauth_state_and_allowlist_rbac() {
    dotenvy::dotenv().ok();
    let db_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be configured in .env or environment to run tests");
    let pool: DbPool = db::init_db(&db_url).await.expect("Failed to init postgres db");

    // 1. Test OAuth State generation and single-use consumption (CSRF protection)
    let (state, nonce) = auth::create_oauth_state(&pool, Some("http://localhost:8081/admin"))
        .await
        .expect("Failed to create OAuth state");
    assert_eq!(state.len(), 64);
    assert_eq!(nonce.len(), 32);

    // Verifying once succeeds
    let redirect = auth::verify_and_consume_oauth_state(&pool, &state)
        .await
        .expect("Verify state failed");
    assert_eq!(redirect.as_deref(), Some("http://localhost:8081/admin"));

    // Verifying again fails because state was consumed (anti-replay)
    let second_try = auth::verify_and_consume_oauth_state(&pool, &state)
        .await
        .expect("Verify second try failed");
    assert!(second_try.is_none());

    // 2. Test Server-side Email Allowlist: Unapproved email is rejected!
    let unauthorized_email = "hacker@randomdomain.xyz";
    let is_allowed = auth::is_email_allowed(&pool, unauthorized_email)
        .await
        .expect("Allowlist check failed");
    assert!(is_allowed.is_none(), "Unauthorized email should be rejected");

    // 3. Test Pre-approved Allowlist Management (Add, Check, Remove)
    let test_admin_email = "approved.educator@example.com";
    auth::add_to_allowlist(&pool, test_admin_email, "admin", Some("Test educator"))
        .await
        .expect("Failed to add to allowlist");

    let allowed_role = auth::is_email_allowed(&pool, test_admin_email)
        .await
        .expect("Check added email failed");
    assert_eq!(allowed_role.as_deref(), Some("admin"), "Pre-approved email must be allowed with correct role");

    // 4. Test Google Admin profile upsert in PostgreSQL
    let google_user = GoogleUserInfo {
        sub: "google_sub_987654321".to_string(),
        email: test_admin_email.to_string(),
        email_verified: Some(true),
        name: Some("Prof. Alex Rivera".to_string()),
        picture: Some("https://example.com/alex.jpg".to_string()),
    };

    let admin = auth::upsert_google_admin(&pool, &google_user, "admin")
        .await
        .expect("Failed to upsert Google admin");

    assert_eq!(admin.email.as_deref(), Some(test_admin_email));
    assert_eq!(admin.name.as_deref(), Some("Prof. Alex Rivera"));
    assert_eq!(admin.avatar_url.as_deref(), Some("https://example.com/alex.jpg"));
    assert_eq!(admin.role, "admin");

    // 5. Test Session creation, validation, and expiry
    let token = auth::create_session_with_meta(
        &pool,
        &admin.id,
        Some("Mozilla/5.0 TestBrowser"),
        Some("127.0.0.1"),
    )
    .await
    .expect("Failed to create session with metadata");

    let validated = auth::validate_session(&pool, &token)
        .await
        .expect("Failed to validate session");
    assert!(validated.is_some());
    let valid_admin = validated.unwrap();
    assert_eq!(valid_admin.id, admin.id);
    assert_eq!(valid_admin.email.as_deref(), Some(test_admin_email));
    assert_eq!(valid_admin.role, "admin");

    // 6. Test Logout / Session Invalidation
    auth::delete_session(&pool, &token).await.expect("Failed to delete session");
    let after_delete = auth::validate_session(&pool, &token)
        .await
        .expect("Validate session failed");
    assert!(after_delete.is_none(), "Session must be invalid after deletion");

    // 7. Cleanup allowlist entry
    let removed = auth::remove_from_allowlist(&pool, test_admin_email)
        .await
        .expect("Failed to remove from allowlist");
    assert!(removed);
}
