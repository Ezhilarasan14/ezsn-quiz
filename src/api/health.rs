use actix_web::{web, HttpResponse, Responder};
use crate::db::DbPool;

pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "ezsn_quiz",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

pub async fn health_db(pool: web::Data<DbPool>) -> impl Responder {
    match sqlx::query("SELECT 1").execute(pool.get_ref()).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "status": "healthy",
            "database": "connected",
            "type": "postgres"
        })),
        Err(e) => {
            log::error!("Database health check failed: {}", e);
            HttpResponse::ServiceUnavailable().json(serde_json::json!({
                "status": "unhealthy",
                "database": "error",
                "error": e.to_string()
            }))
        }
    }
}
