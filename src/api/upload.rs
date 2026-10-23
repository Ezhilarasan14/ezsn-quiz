use actix_web::{http::header, web, HttpRequest, HttpResponse, Responder};
use base64::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct UploadRequest {
    pub filename: Option<String>,
    pub data: String, // Base64 data URL or pure base64
}

#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub url: String,
    pub filename: String,
}

pub fn get_upload_dir() -> PathBuf {
    let dir = env::var("UPLOAD_DIR").unwrap_or_else(|_| "./uploads".to_string());
    let path = PathBuf::from(dir);
    if !path.exists() {
        let _ = fs::create_dir_all(&path);
    }
    path
}

pub async fn upload_file(body: web::Json<UploadRequest>) -> impl Responder {
    let raw_data = &body.data;

    // Detect format and extract pure base64
    let (mime_ext, b64_str) = if let Some(comma_pos) = raw_data.find(',') {
        let prefix = &raw_data[..comma_pos];
        let ext = if prefix.contains("image/png") {
            "png"
        } else if prefix.contains("image/jpeg") || prefix.contains("image/jpg") {
            "jpg"
        } else if prefix.contains("image/svg+xml") {
            "svg"
        } else if prefix.contains("image/webp") {
            "webp"
        } else if prefix.contains("image/gif") {
            "gif"
        } else {
            "png"
        };
        (ext, &raw_data[comma_pos + 1..])
    } else {
        ("png", raw_data.as_str())
    };

    let bytes = match BASE64_STANDARD.decode(b64_str.trim()) {
        Ok(b) => b,
        Err(e) => {
            log::error!("Failed to decode base64 file data: {}", e);
            return HttpResponse::BadRequest().json(json!({
                "error": "Invalid base64 payload"
            }));
        }
    };

    if bytes.len() > 10 * 1024 * 1024 {
        return HttpResponse::BadRequest().json(json!({
            "error": "File size exceeds 10MB limit"
        }));
    }

    let upload_dir = get_upload_dir();
    let unique_name = format!("{}.{}", Uuid::new_v4(), mime_ext);
    let target_path = upload_dir.join(&unique_name);

    if let Err(e) = fs::write(&target_path, bytes) {
        log::error!("Failed to save uploaded file to '{:?}': {}", target_path, e);
        return HttpResponse::InternalServerError().json(json!({
            "error": "Failed to save file"
        }));
    }

    let public_url = format!("/uploads/{}", unique_name);
    log::info!("Uploaded asset saved successfully: {}", public_url);

    HttpResponse::Ok().json(UploadResponse {
        url: public_url,
        filename: unique_name,
    })
}

pub async fn serve_upload(_req: HttpRequest, path: web::Path<String>) -> impl Responder {
    let filename = path.into_inner();
    let upload_dir = get_upload_dir();
    let file_path = upload_dir.join(&filename);

    if !file_path.exists() || !file_path.is_file() {
        return HttpResponse::NotFound().body("Asset Not Found");
    }

    match fs::read(&file_path) {
        Ok(data) => {
            let mime = mime_guess::from_path(&file_path).first_or_octet_stream();
            HttpResponse::Ok()
                .insert_header((header::CONTENT_TYPE, mime.as_ref()))
                .insert_header((header::CACHE_CONTROL, "public, max-age=86400"))
                .body(data)
        }
        Err(e) => {
            log::error!("Error reading upload file '{:?}': {}", file_path, e);
            HttpResponse::InternalServerError().body("Error reading asset")
        }
    }
}
