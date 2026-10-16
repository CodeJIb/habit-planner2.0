use sqlx::SqliteConnection;

use crate::errors::AppError;
use crate::models::category::{Category, CategoryInput};

pub async fn list(conn: &mut SqliteConnection) -> Result<Vec<Category>, AppError> {
    let rows = sqlx::query_as::<_, Category>("SELECT * FROM categories ORDER BY name")
        .fetch_all(&mut *conn)
        .await?;
    Ok(rows)
}

pub async fn insert(conn: &mut SqliteConnection, input: &CategoryInput) -> Result<Category, AppError> {
    let row = sqlx::query_as::<_, Category>(
        "INSERT INTO categories (name, color) VALUES (?, ?) RETURNING *",
    )
    .bind(&input.name)
    .bind(&input.color)
    .fetch_one(&mut *conn)
    .await?;
    Ok(row)
}

pub async fn delete(conn: &mut SqliteConnection, id: i64) -> Result<bool, AppError> {
    let result = sqlx::query("DELETE FROM categories WHERE id = ?")
        .bind(id)
        .execute(&mut *conn)
        .await?;
    Ok(result.rows_affected() > 0)
}
