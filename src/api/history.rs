use actix_web::{web, HttpResponse, Responder};
use crate::db::{self, DbPool};

pub async fn list_history(pool: web::Data<DbPool>) -> impl Responder {
    match db::list_game_history(pool.get_ref()).await {
        Ok(history) => HttpResponse::Ok().json(history),
        Err(e) => {
            log::error!("Failed to fetch game history: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({ "error": "Failed to fetch game history" }))
        }
    }
}

pub async fn get_game_results(pool: web::Data<DbPool>, path: web::Path<String>) -> impl Responder {
    let game_id = path.into_inner();
    match db::get_game_results(pool.get_ref(), &game_id).await {
        Ok(results) => HttpResponse::Ok().json(results),
        Err(e) => {
            log::error!("Failed to fetch results for game {}: {}", game_id, e);
            HttpResponse::InternalServerError().json(serde_json::json!({ "error": "Failed to fetch game results" }))
        }
    }
}

pub async fn get_player_stats(pool: web::Data<DbPool>, path: web::Path<String>) -> impl Responder {
    let email = path.into_inner();
    match db::get_player_stats(pool.get_ref(), &email).await {
        Ok(Some(stats)) => HttpResponse::Ok().json(stats),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({ "error": "No stats found for this email" })),
        Err(e) => {
            log::error!("Failed to fetch player stats for {}: {}", email, e);
            HttpResponse::InternalServerError().json(serde_json::json!({ "error": "Failed to fetch player stats" }))
        }
    }
}
