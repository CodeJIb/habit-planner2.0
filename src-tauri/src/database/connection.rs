use std::path::Path;

use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use sqlx::SqlitePool;

use crate::errors::AppError;

/// Open the database (creating it if needed), enforce foreign keys, and apply migrations.
pub async fn init_pool(db_path: &Path) -> Result<SqlitePool, AppError> {
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let options = SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(true)
        .foreign_keys(true) // applied to every pooled connection
        .journal_mode(SqliteJournalMode::Wal);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    // Migrations are embedded into the binary at compile time from src-tauri/migrations/.
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}
