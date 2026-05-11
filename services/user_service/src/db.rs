use once_cell::sync::Lazy;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::{fs, path::Path};

/// Lazily initialized global DB pool.
/// Set `DATABASE_URL` to override the default path.
pub static DB_POOL: Lazy<SqlitePool> = Lazy::new(|| {
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite://data/users.db?mode=rwc".into());

    // Extract the file path and ensure its parent directory exists
    if let Some(path) = db_url.strip_prefix("sqlite://") {
        let path = path.split('?').next().unwrap_or(path);
        if let Some(parent) = Path::new(path).parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).expect("❌ Failed to create database directory");
            }
        }
    }

    SqlitePoolOptions::new()
        .max_connections(5)
        .connect_lazy(&db_url)
        .expect("❌ Failed to connect to SQLite")
});

/// Initializes the users table if it does not exist
pub async fn init() {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            email TEXT NOT NULL
        )",
    )
    .execute(&*DB_POOL)
    .await
    .expect("❌ Failed to initialize the users table");
}
