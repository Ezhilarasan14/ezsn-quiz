pub mod auth;
pub mod health;
pub mod history;
pub mod organization;
pub mod quizzes;
pub mod rooms;
pub mod upload;

use actix_web::web;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            // Health
            .route("/health", web::get().to(health::health_check))
            .route("/health/db", web::get().to(health::health_db))
            // Auth
            .route("/auth/login", web::post().to(auth::login))
            .route("/auth/logout", web::post().to(auth::logout))
            .route("/auth/me", web::get().to(auth::me))
            // Organization & Branding
            .route("/organization", web::get().to(organization::get_org))
            .route("/organization", web::put().to(organization::update_org))
            // Uploads
            .route("/upload", web::post().to(upload::upload_file))
            // Quizzes CRUD
            .route("/quizzes", web::get().to(quizzes::list_quizzes))
            .route("/quizzes", web::post().to(quizzes::create_quiz))
            .route("/quizzes/{id}", web::get().to(quizzes::get_quiz))
            .route("/quizzes/{id}", web::put().to(quizzes::update_quiz))
            .route("/quizzes/{id}", web::delete().to(quizzes::delete_quiz))
            .route("/quizzes/{id}/duplicate", web::post().to(quizzes::duplicate_quiz))
            // Rooms
            .route("/rooms", web::post().to(rooms::create_room))
            .route("/rooms/{pin}", web::get().to(rooms::get_room_status))
            // History & Player Analytics
            .route("/history", web::get().to(history::list_history))
            .route("/history/{game_id}", web::get().to(history::get_game_results))
            .route("/players/{email}/stats", web::get().to(history::get_player_stats)),
    )
    .route("/health", web::get().to(health::health_check))
    .route("/health/db", web::get().to(health::health_db))
    .route("/uploads/{filename}", web::get().to(upload::serve_upload));
}
