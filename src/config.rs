use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub cleanup_interval_secs: u64,
    pub room_abandoned_timeout_secs: u64,
    pub max_room_lifetime_secs: u64,
    pub quiz_question_time_seconds: u64,
}

impl Config {
    pub fn from_env() -> Self {
        let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8081);
        let database_url = env::var("DATABASE_URL").unwrap_or_default();

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

        let quiz_question_time_seconds = env::var("QUIZ_QUESTION_TIME_SECONDS")
            .ok()
            .and_then(|v| v.trim().parse::<u64>().ok())
            .unwrap_or(5);

        Self {
            host,
            port,
            database_url,
            cleanup_interval_secs,
            room_abandoned_timeout_secs,
            max_room_lifetime_secs,
            quiz_question_time_seconds,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.quiz_question_time_seconds == 0 {
            return Err("QUIZ_QUESTION_TIME_SECONDS must be at least 1 second".to_string());
        }
        if self.quiz_question_time_seconds > 300 {
            return Err("QUIZ_QUESTION_TIME_SECONDS cannot exceed 300 seconds (5 minutes)".to_string());
        }
        if self.port == 0 {
            return Err("PORT must be a valid non-zero port number".to_string());
        }
        if self.database_url.trim().is_empty() {
            return Err("DATABASE_URL environment variable is required. Please set it in your .env file or environment (e.g. DATABASE_URL=postgres://username:password@localhost:5432/database_name)".to_string());
        }
        Ok(())
    }

    /// Returns the database URL with the password masked for safe logging
    pub fn masked_database_url(&self) -> String {
        if let Some(at_idx) = self.database_url.find('@') {
            let scheme_and_user_part = &self.database_url[..at_idx];
            let host_and_path_part = &self.database_url[at_idx..];
            if let Some(colon_idx) = scheme_and_user_part.rfind(':') {
                if let Some(slash_idx) = scheme_and_user_part.find("://") {
                    if colon_idx > slash_idx + 2 {
                        let prefix = &scheme_and_user_part[..colon_idx];
                        return format!("{}:******{}", prefix, host_and_path_part);
                    }
                }
            }
        }
        if self.database_url.contains("://") {
            "postgres://******".to_string()
        } else {
            "******".to_string()
        }
    }
}
