use sqlx::SqliteConnection;

use crate::errors::AppError;
use crate::models::task::{Task, TaskInput};

// SQL fragment for "now" in the same ISO format the schema uses for defaults.
const NOW: &str = "strftime('%Y-%m-%dT%H:%M:%SZ','now')";

pub async fn insert(conn: &mut SqliteConnection, input: &TaskInput) -> Result<Task, AppError> {
    let task = sqlx::query_as::<_, Task>(
        "INSERT INTO tasks (title, description, priority, category_id, due_at, scheduled_start_at,
                            estimated_duration_minutes, recurrence_rule)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)
         RETURNING *",
    )
    .bind(&input.title)
    .bind(&input.description)
    .bind(&input.priority)
    .bind(input.category_id)
    .bind(&input.due_at)
    .bind(&input.scheduled_start_at)
    .bind(input.estimated_duration_minutes)
    .bind(&input.recurrence_rule)
    .fetch_one(&mut *conn)
    .await?;
    Ok(task)
}

pub async fn find(conn: &mut SqliteConnection, id: i64) -> Result<Option<Task>, AppError> {
    let task = sqlx::query_as::<_, Task>("SELECT * FROM tasks WHERE id = ?")
        .bind(id)
        .fetch_optional(&mut *conn)
        .await?;
    Ok(task)
}

pub async fn list(conn: &mut SqliteConnection, status: Option<&str>) -> Result<Vec<Task>, AppError> {
    let tasks = match status {
        Some(s) => {
            sqlx::query_as::<_, Task>("SELECT * FROM tasks WHERE status = ? ORDER BY created_at DESC, id DESC")
                .bind(s)
                .fetch_all(&mut *conn)
                .await?
        }
        None => {
            sqlx::query_as::<_, Task>("SELECT * FROM tasks ORDER BY created_at DESC, id DESC")
                .fetch_all(&mut *conn)
                .await?
        }
    };
    Ok(tasks)
}

pub async fn update(
    conn: &mut SqliteConnection,
    id: i64,
    input: &TaskInput,
) -> Result<Option<Task>, AppError> {
    let sql = format!(
        "UPDATE tasks
         SET title = ?, description = ?, priority = ?, category_id = ?, due_at = ?,
             scheduled_start_at = ?, estimated_duration_minutes = ?, recurrence_rule = ?,
             updated_at = {NOW}
         WHERE id = ?
         RETURNING *"
    );
    let task = sqlx::query_as::<_, Task>(&sql)
        .bind(&input.title)
        .bind(&input.description)
        .bind(&input.priority)
        .bind(input.category_id)
        .bind(&input.due_at)
        .bind(&input.scheduled_start_at)
        .bind(input.estimated_duration_minutes)
        .bind(&input.recurrence_rule)
        .bind(id)
        .fetch_optional(&mut *conn)
        .await?;
    Ok(task)
}

/// Change status and keep `completed_at` / `archived_at` consistent with it.
pub async fn set_status(
    conn: &mut SqliteConnection,
    id: i64,
    status: &str,
) -> Result<Option<Task>, AppError> {
    let sql = format!(
        "UPDATE tasks
         SET status = ?1,
             updated_at   = {NOW},
             completed_at = CASE WHEN ?1 = 'done'     THEN {NOW} ELSE NULL END,
             archived_at  = CASE WHEN ?1 = 'archived' THEN {NOW} ELSE NULL END
         WHERE id = ?2
         RETURNING *"
    );
    let task = sqlx::query_as::<_, Task>(&sql)
        .bind(status)
        .bind(id)
        .fetch_optional(&mut *conn)
        .await?;
    Ok(task)
}

pub async fn delete(conn: &mut SqliteConnection, id: i64) -> Result<bool, AppError> {
    let result = sqlx::query("DELETE FROM tasks WHERE id = ?")
        .bind(id)
        .execute(&mut *conn)
        .await?;
    Ok(result.rows_affected() > 0)
}
