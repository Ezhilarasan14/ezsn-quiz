use actix_web::{web, HttpRequest, HttpResponse, Responder};
use serde_json::json;

use crate::api::auth::extract_token;
use crate::auth;
use crate::db::models::UpdateOrganizationRequest;
use crate::db::organizations::{get_default_organization, update_organization as db_update_org};
use crate::db::DbPool;

pub async fn get_org(pool: web::Data<DbPool>) -> impl Responder {
    match get_default_organization(pool.get_ref()).await {
        Ok(org) => HttpResponse::Ok().json(org),
        Err(e) => {
            log::error!("Failed to fetch organization: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "error": "Failed to fetch organization branding"
            }))
        }
    }
}

pub async fn update_org(
    pool: web::Data<DbPool>,
    req: HttpRequest,
    body: web::Json<UpdateOrganizationRequest>,
) -> impl Responder {
    let token = match extract_token(&req) {
        Some(t) => t,
        None => {
            return HttpResponse::Unauthorized().json(json!({
                "error": "Unauthorized"
            }));
        }
    };

    let admin = match auth::validate_session(pool.get_ref(), &token).await {
        Ok(Some(a)) => a,
        _ => {
            return HttpResponse::Unauthorized().json(json!({
                "error": "Invalid or expired session"
            }));
        }
    };

    let org_id = admin.organization_id.unwrap_or_else(|| "org_default".to_string());

    match db_update_org(pool.get_ref(), &org_id, body.into_inner()).await {
        Ok(updated) => HttpResponse::Ok().json(updated),
        Err(e) => {
            log::error!("Failed to update organization branding: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "error": "Failed to update branding"
            }))
        }
    }
}
