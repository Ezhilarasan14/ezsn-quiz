use actix_web::{http::header, HttpRequest, HttpResponse, Responder};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "frontend/dist/"]
pub struct Asset;

pub async fn serve_static(req: HttpRequest) -> impl Responder {
    let path = req.path().trim_start_matches('/').to_string();

    // Check if the requested exact asset exists
    if !path.is_empty() {
        if let Some(content) = Asset::get(&path) {
            let mime = mime_guess::from_path(&path).first_or_octet_stream();
            return HttpResponse::Ok()
                .insert_header((header::CONTENT_TYPE, mime.as_ref()))
                .insert_header((header::CACHE_CONTROL, "public, max-age=3600"))
                .body(content.data.into_owned());
        }
    }

    // SPA fallback: serve index.html for client-side routing
    match Asset::get("index.html") {
        Some(content) => HttpResponse::Ok()
            .insert_header((header::CONTENT_TYPE, "text/html; charset=utf-8"))
            .insert_header((header::CACHE_CONTROL, "no-cache"))
            .body(content.data.into_owned()),
        None => HttpResponse::NotFound().body("404 Not Found"),
    }
}
