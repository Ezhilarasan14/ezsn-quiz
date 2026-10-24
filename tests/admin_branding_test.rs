use ezsn_quiz::auth;
use ezsn_quiz::db::{self, DbPool};
use ezsn_quiz::db::models::{CreateOptionRequest, CreateQuestionRequest, CreateQuizRequest, UpdateOrganizationRequest};

#[tokio::test]
async fn test_admin_auth_and_branding_system() {
    dotenvy::dotenv().ok();
    let db_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be configured in .env or environment to run tests");
    let pool: DbPool = db::init_db(&db_url).await.expect("Failed to init postgres db");

    // 1. Verify default admin initialized
    auth::ensure_default_admin(&pool).await.expect("Failed to ensure default admin");
    
    // 2. Test password verification
    let (salt, hash) = auth::hash_password("admin123");
    assert!(auth::verify_password("admin123", &salt, &hash));
    assert!(!auth::verify_password("wrongpass", &salt, &hash));

    // 3. Test session creation and validation
    let admin = sqlx::query_as::<_, ezsn_quiz::db::models::AdminUser>(
        "SELECT id, organization_id, username FROM admins WHERE username = 'admin'"
    )
    .fetch_one(&pool)
    .await
    .expect("Admin should exist");

    let token = auth::create_session(&pool, &admin.id).await.expect("Failed to create session");
    assert_eq!(token.len(), 64);

    let validated = auth::validate_session(&pool, &token).await.expect("Validate session query failed");
    assert!(validated.is_some());
    assert_eq!(validated.unwrap().username, "admin");

    // 4. Test organization branding retrieval and update
    let default_org = db::get_default_organization(&pool).await.expect("Failed to get default org");
    assert!(!default_org.id.is_empty());
    assert!(!default_org.name.is_empty());

    let updated_org = db::update_organization(
        &pool,
        &default_org.id,
        UpdateOrganizationRequest {
            name: "Acme High School".to_string(),
            branch_name: Some("Downtown Campus".to_string()),
            logo_url: Some("/uploads/acme_logo.png".to_string()),
            primary_color: Some("#10b981".to_string()),
            secondary_color: Some("#06b6d4".to_string()),
            background_color: Some("#064e3b".to_string()),
            button_color: Some("#10b981".to_string()),
        },
    )
    .await
    .expect("Failed to update organization");

    assert_eq!(updated_org.name, "Acme High School");
    assert_eq!(updated_org.branch_name.as_deref(), Some("Downtown Campus"));
    assert_eq!(updated_org.logo_url.as_deref(), Some("/uploads/acme_logo.png"));
    assert_eq!(updated_org.primary_color, "#10b981");

    // 5. Test quiz creation with quiz-specific theme override
    let quiz_id = db::create_quiz(
        &pool,
        CreateQuizRequest {
            title: "Physics Championship".to_string(),
            description: Some("Branded quiz".to_string()),
            theme_logo_url: Some("/uploads/physics_logo.png".to_string()),
            theme_primary_color: Some("#f97316".to_string()),
            theme_secondary_color: Some("#e11d48".to_string()),
            theme_bg_color: Some("#431407".to_string()),
            theme_btn_color: Some("#ea580c".to_string()),
            questions: vec![CreateQuestionRequest {
                question_text: "What is Newton's second law?".to_string(),
                time_limit_sec: Some(20),
                points: Some(1000),
                options: vec![
                    CreateOptionRequest { option_text: "F = ma".to_string(), is_correct: true },
                    CreateOptionRequest { option_text: "E = mc^2".to_string(), is_correct: false },
                ],
            }],
            ..Default::default()
        },
    )
    .await
    .expect("Failed to create quiz with custom branding");

    let fetched_quiz = db::get_quiz_by_id(&pool, &quiz_id).await.unwrap().unwrap();
    assert_eq!(fetched_quiz.theme_logo_url.as_deref(), Some("/uploads/physics_logo.png"));
    assert_eq!(fetched_quiz.theme_primary_color.as_deref(), Some("#f97316"));
    assert_eq!(fetched_quiz.theme_secondary_color.as_deref(), Some("#e11d48"));
    assert_eq!(fetched_quiz.theme_bg_color.as_deref(), Some("#431407"));
    assert_eq!(fetched_quiz.theme_btn_color.as_deref(), Some("#ea580c"));

    // Clean up test quiz
    let _ = db::delete_quiz(&pool, &quiz_id).await;
}
