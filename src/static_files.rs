use actix_web::{http::header, HttpRequest, HttpResponse, Responder};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "static/"]
pub struct Asset;

pub async fn serve_static(req: HttpRequest) -> impl Responder {
    let mut path = req.path().trim_start_matches('/').to_string();

    if path.is_empty() || path == "index.html" {
        path = "index.html".to_string();
    } else if path == "host" {
        path = "host.html".to_string();
    } else if path == "play" {
        path = "play.html".to_string();
    } else if path == "edit" {
        path = "edit.html".to_string();
    } else if path == "history" {
        path = "history.html".to_string();
    }

    match Asset::get(&path) {
        Some(content) => {
            let mime = mime_guess::from_path(&path).first_or_octet_stream();
            HttpResponse::Ok()
                .insert_header((header::CONTENT_TYPE, mime.as_ref()))
                .insert_header((header::CACHE_CONTROL, "public, max-age=3600"))
                .body(content.data.into_owned())
        }
        None => HttpResponse::NotFound().body("404 Not Found"),
    }
}
