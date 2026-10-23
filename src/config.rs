use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
}

impl Config {
    pub fn from_env() -> Self {
        let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8080);
        let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:quiz.db".to_string());

        Self {
            host,
            port,
            database_url,
        }
    }
}
