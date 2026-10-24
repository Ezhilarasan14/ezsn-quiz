use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::api::auth::AuthenticatedAdmin;
use crate::db::{self, DbPool};
use crate::game::manager::RoomManager;
use crate::game::types::GameStateKind;

#[derive(Debug, Deserialize)]
pub struct CreateRoomRequest {
    pub quiz_id: String,
}

#[derive(Debug, Serialize)]
pub struct CreateRoomResponse {
    pub pin: String,
    pub host_token: String,
    pub game_id: String,
    pub quiz_title: String,
}

#[derive(Debug, Serialize)]
pub struct RoomStatusResponse {
    pub pin: String,
    pub quiz_title: String,
    pub status: GameStateKind,
    pub player_count: usize,
    pub total_questions: usize,
}

pub async fn create_room(
    pool: web::Data<DbPool>,
    manager: web::Data<Arc<RoomManager>>,
    _admin: AuthenticatedAdmin,
    body: web::Json<CreateRoomRequest>,
) -> impl Responder {
    let quiz_id = &body.quiz_id;

    let quiz = match db::get_quiz_by_id(pool.get_ref(), quiz_id).await {
        Ok(Some(q)) => q,
        Ok(None) => return HttpResponse::NotFound().json(serde_json::json!({ "error": "Quiz not found" })),
        Err(e) => {
            log::error!("Error fetching quiz {}: {}", quiz_id, e);
            return HttpResponse::InternalServerError().json(serde_json::json!({ "error": "Failed to fetch quiz" }));
        }
    };

    if quiz.questions.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Cannot host a quiz with no questions. Please add questions first."
        }));
    }

    let title = quiz.title.clone();
    match manager.create_room(quiz).await {
        Ok((pin, host_token, game_id)) => HttpResponse::Created().json(CreateRoomResponse {
            pin,
            host_token,
            game_id,
            quiz_title: title,
        }),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": err })),
    }
}

pub async fn get_room_status(
    manager: web::Data<Arc<RoomManager>>,
    path: web::Path<String>,
) -> impl Responder {
    let pin = path.into_inner();
    let room_arc = match manager.get_room(&pin).await {
        Some(r) => r,
        None => return HttpResponse::NotFound().json(serde_json::json!({ "error": "Room not found or expired" })),
    };

    let room = room_arc.read().await;
    let status = match room.state {
        crate::game::types::RoomState::Lobby => GameStateKind::Lobby,
        crate::game::types::RoomState::QuestionIntro { .. } => GameStateKind::QuestionIntro,
        crate::game::types::RoomState::QuestionActive { .. } => GameStateKind::QuestionActive,
        crate::game::types::RoomState::QuestionResult { .. } => GameStateKind::QuestionResult,
        crate::game::types::RoomState::Leaderboard { .. } => GameStateKind::Leaderboard,
        crate::game::types::RoomState::Podium { .. } => GameStateKind::Podium,
    };

    HttpResponse::Ok().json(RoomStatusResponse {
        pin: room.pin.clone(),
        quiz_title: room.quiz.title.clone(),
        status,
        player_count: room.active_player_count(),
        total_questions: room.quiz.questions.len(),
    })
}
