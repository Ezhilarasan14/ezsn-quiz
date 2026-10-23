pub mod games;
pub mod models;
pub mod organizations;
pub mod quizzes;

pub use games::*;
pub use organizations::*;
pub use quizzes::*;

use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::time::Duration;

pub type DbPool = Pool<Postgres>;

pub async fn init_db(database_url: &str) -> Result<DbPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(15))
        .connect(database_url)
        .await?;

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

    log::info!("PostgreSQL database initialized and migrations applied successfully.");
    Ok(pool)
}
