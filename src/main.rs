use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpServer};
use std::sync::Arc;

use ezsn_quiz::api;
use ezsn_quiz::auth;
use ezsn_quiz::config::Config;
use ezsn_quiz::db;
use ezsn_quiz::game::RoomManager;
use ezsn_quiz::static_files;
use ezsn_quiz::ws;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let config = Config::from_env();
    log::info!("Starting ezsn-quiz server configuration: host={}, port={}, db={}", config.host, config.port, config.database_url);

    // Initialize database pool and run migrations with detailed error logging
    let pool = match db::init_db(&config.database_url).await {
        Ok(p) => p,
        Err(e) => {
            log::error!("CRITICAL: Failed to initialize database at '{}': {}", config.database_url, e);
            eprintln!("CRITICAL: Failed to initialize database at '{}': {}", config.database_url, e);
            std::process::exit(1);
        }
    };

    if let Err(e) = auth::ensure_default_admin(&pool).await {
        log::warn!("Could not verify default admin account: {}", e);
    }

    // Initialize in-memory Game Room Manager
    let manager = Arc::new(RoomManager::new(pool.clone()));

    let bind_addr = format!("{}:{}", config.host, config.port);
    log::info!("Server listening on http://{}", bind_addr);

    let server = match HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(middleware::Logger::default())
            .wrap(cors)
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(manager.clone()))
            // WebSocket endpoint
            .route("/ws", web::get().to(ws::ws_route))
            // REST API endpoints
            .configure(api::configure_routes)
            // Static files and SPA fallback
            .default_service(web::to(static_files::serve_static))
    })
    .bind(&bind_addr) {
        Ok(s) => s,
        Err(e) => {
            log::error!("CRITICAL: Failed to bind server to '{}': {}", bind_addr, e);
            eprintln!("CRITICAL: Failed to bind server to '{}': {}", bind_addr, e);
            std::process::exit(1);
        }
    };

    server.run().await
}
