use sqlx::SqliteConnection;

use crate::errors::AppError;

// SQL fragment for "now" in the same ISO format the schema uses for defaults.
const NOW: &str = "strftime('%Y-%m-%dT%H:%M:%SZ','now')";

// Generic key/value helpers over `app_settings`, shared by any feature that
// keeps a small choice in the database (assistant credentials, reminders, ...).

pub async fn get_setting(conn: &mut SqliteConnection, key: &str) -> Result<Option<String>, AppError> {
    let value = sqlx::query_scalar::<_, String>("SELECT value FROM app_settings WHERE key = ?")
        .bind(key)
        .fetch_optional(&mut *conn)
        .await?;
    Ok(value)
}

pub async fn set_setting(
    conn: &mut SqliteConnection,
    key: &str,
    value: &str,
) -> Result<(), AppError> {
    let sql = format!(
        "INSERT INTO app_settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = ?2, updated_at = {NOW}"
    );
    sqlx::query(&sql)
        .bind(key)
        .bind(value)
        .execute(&mut *conn)
        .await?;
    Ok(())
}

pub async fn delete_setting(conn: &mut SqliteConnection, key: &str) -> Result<(), AppError> {
    sqlx::query("DELETE FROM app_settings WHERE key = ?")
        .bind(key)
        .execute(&mut *conn)
        .await?;
    Ok(())
}
