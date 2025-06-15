use once_cell::sync::Lazy;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::{fs, path::Path};

const DB_URL: &str = "sqlite://data/users.db";

/// Lazily initialized global DB pool
pub static DB_POOL: Lazy<SqlitePool> = Lazy::new(|| {
    // Ensure the `data` directory exists relative to the executable
    let db_dir = Path::new("data");
    if !db_dir.exists() {
        fs::create_dir_all(db_dir).expect("❌ Failed to create 'data/' directory");
    }

    SqlitePoolOptions::new()
        .max_connections(5)
        .connect_lazy(DB_URL)
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
