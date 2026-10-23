pub mod history;
pub mod quizzes;
pub mod rooms;

use actix_web::web;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
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
            // History
            .route("/history", web::get().to(history::list_history))
            .route("/history/{game_id}", web::get().to(history::get_game_results)),
    );
}
