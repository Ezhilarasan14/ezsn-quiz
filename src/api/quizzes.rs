use actix_web::{web, HttpResponse, Responder};
use crate::api::auth::AuthenticatedAdmin;
use crate::db::{self, models::*, DbPool};

pub async fn list_quizzes(pool: web::Data<DbPool>) -> impl Responder {
    match db::list_quizzes(pool.get_ref()).await {
        Ok(quizzes) => HttpResponse::Ok().json(quizzes),
        Err(e) => {
            log::error!("Failed to fetch quizzes: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({ "error": "Failed to fetch quizzes" }))
        }
    }
}

pub async fn get_quiz(pool: web::Data<DbPool>, path: web::Path<String>) -> impl Responder {
    let quiz_id = path.into_inner();
    match db::get_quiz_by_id(pool.get_ref(), &quiz_id).await {
        Ok(Some(quiz)) => HttpResponse::Ok().json(quiz),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({ "error": "Quiz not found" })),
        Err(e) => {
            log::error!("Failed to get quiz {}: {}", quiz_id, e);
            HttpResponse::InternalServerError().json(serde_json::json!({ "error": "Failed to fetch quiz" }))
        }
    }
}

pub async fn create_quiz(
    pool: web::Data<DbPool>,
    _admin: AuthenticatedAdmin,
    body: web::Json<CreateQuizRequest>,
) -> impl Responder {
    let req = body.into_inner();

    if req.title.trim().is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({ "error": "Quiz title cannot be empty" }));
    }

    if req.questions.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({ "error": "Quiz must contain at least one question" }));
    }

    for (idx, q) in req.questions.iter().enumerate() {
        if q.question_text.trim().is_empty() {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": format!("Question {} text cannot be empty", idx + 1)
            }));
        }

        if q.options.len() < 2 {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": format!("Question {} must have at least 2 options", idx + 1)
            }));
        }

        let has_correct = q.options.iter().any(|o| o.is_correct);
        if !has_correct {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": format!("Question {} must have at least one correct answer selected", idx + 1)
            }));
        }
    }

    match db::create_quiz(pool.get_ref(), req).await {
        Ok(id) => HttpResponse::Created().json(serde_json::json!({ "id": id, "message": "Quiz created successfully" })),
        Err(e) => {
            log::error!("Failed to create quiz: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({ "error": "Failed to create quiz" }))
        }
    }
}

pub async fn update_quiz(
    pool: web::Data<DbPool>,
    _admin: AuthenticatedAdmin,
    path: web::Path<String>,
    body: web::Json<UpdateQuizRequest>,
) -> impl Responder {
    let quiz_id = path.into_inner();
    let req = body.into_inner();

    if req.title.trim().is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({ "error": "Quiz title cannot be empty" }));
    }

    if req.questions.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({ "error": "Quiz must contain at least one question" }));
    }

    for (idx, q) in req.questions.iter().enumerate() {
        if q.question_text.trim().is_empty() {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": format!("Question {} text cannot be empty", idx + 1)
            }));
        }

        if q.options.len() < 2 {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": format!("Question {} must have at least 2 options", idx + 1)
            }));
        }

        let has_correct = q.options.iter().any(|o| o.is_correct);
        if !has_correct {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": format!("Question {} must have at least one correct answer selected", idx + 1)
            }));
        }
    }

    match db::update_quiz(pool.get_ref(), &quiz_id, req).await {
        Ok(true) => {
            // Return the full updated quiz so the frontend can update its state
            match db::get_quiz_by_id(pool.get_ref(), &quiz_id).await {
                Ok(Some(updated_quiz)) => HttpResponse::Ok().json(updated_quiz),
                _ => HttpResponse::Ok().json(serde_json::json!({ "id": quiz_id, "message": "Quiz updated successfully" })),
            }
        }
        Ok(false) => HttpResponse::NotFound().json(serde_json::json!({ "error": "Quiz not found" })),
        Err(e) => {
            log::error!("Failed to update quiz {}: {}", quiz_id, e);
            HttpResponse::InternalServerError().json(serde_json::json!({ "error": "Failed to update quiz" }))
        }
    }
}

pub async fn delete_quiz(
    pool: web::Data<DbPool>,
    _admin: AuthenticatedAdmin,
    path: web::Path<String>,
) -> impl Responder {
    let quiz_id = path.into_inner();
    match db::delete_quiz(pool.get_ref(), &quiz_id).await {
        Ok(true) => HttpResponse::Ok().json(serde_json::json!({ "message": "Quiz deleted successfully" })),
        Ok(false) => HttpResponse::NotFound().json(serde_json::json!({ "error": "Quiz not found" })),
        Err(e) => {
            log::error!("Failed to delete quiz {}: {}", quiz_id, e);
            HttpResponse::InternalServerError().json(serde_json::json!({ "error": "Failed to delete quiz" }))
        }
    }
}

pub async fn duplicate_quiz(
    pool: web::Data<DbPool>,
    _admin: AuthenticatedAdmin,
    path: web::Path<String>,
) -> impl Responder {
    let quiz_id = path.into_inner();
    match db::duplicate_quiz(pool.get_ref(), &quiz_id).await {
        Ok(Some(new_id)) => HttpResponse::Created().json(serde_json::json!({
            "id": new_id,
            "message": "Quiz duplicated successfully"
        })),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({ "error": "Quiz not found" })),
        Err(e) => {
            log::error!("Failed to duplicate quiz {}: {}", quiz_id, e);
            HttpResponse::InternalServerError().json(serde_json::json!({ "error": "Failed to duplicate quiz" }))
        }
    }
}
