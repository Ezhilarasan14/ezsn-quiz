use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub cleanup_interval_secs: u64,
    pub room_abandoned_timeout_secs: u64,
    pub max_room_lifetime_secs: u64,
}

impl Config {
    pub fn from_env() -> Self {
        let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8081);
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/esan_quiz".to_string());

        let cleanup_interval_secs = env::var("CLEANUP_INTERVAL_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(60);

        let room_abandoned_timeout_secs = env::var("ROOM_ABANDONED_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(900);

        let max_room_lifetime_secs = env::var("MAX_ROOM_LIFETIME_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(7200);

        Self {
            host,
            port,
            database_url,
            cleanup_interval_secs,
            room_abandoned_timeout_secs,
            max_room_lifetime_secs,
        }
    }
}
