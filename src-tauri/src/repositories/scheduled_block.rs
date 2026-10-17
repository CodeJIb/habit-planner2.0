use sqlx::SqliteConnection;

use crate::errors::AppError;
use crate::models::scheduled_block::{ScheduledBlock, ScheduledBlockInput};

pub async fn insert(
    conn: &mut SqliteConnection,
    input: &ScheduledBlockInput,
    block_type: &str,
) -> Result<ScheduledBlock, AppError> {
    let block = sqlx::query_as::<_, ScheduledBlock>(
        "INSERT INTO scheduled_blocks (title, start_at, end_at, task_id, block_type)
         VALUES (?, ?, ?, ?, ?)
         RETURNING *",
    )
    .bind(&input.title)
    .bind(&input.start_at)
    .bind(&input.end_at)
    .bind(input.task_id)
    .bind(block_type)
    .fetch_one(&mut *conn)
    .await?;
    Ok(block)
}

pub async fn find(conn: &mut SqliteConnection, id: i64) -> Result<Option<ScheduledBlock>, AppError> {
    let block = sqlx::query_as::<_, ScheduledBlock>("SELECT * FROM scheduled_blocks WHERE id = ?")
        .bind(id)
        .fetch_optional(&mut *conn)
        .await?;
    Ok(block)
}

pub async fn list_all(conn: &mut SqliteConnection) -> Result<Vec<ScheduledBlock>, AppError> {
    let blocks = sqlx::query_as::<_, ScheduledBlock>(
        "SELECT * FROM scheduled_blocks ORDER BY start_at, id",
    )
    .fetch_all(&mut *conn)
    .await?;
    Ok(blocks)
}

/// Blocks overlapping the closed range [`start_bound`, `end_bound`],
/// where both are local datetimes "YYYY-MM-DDTHH:MM".
pub async fn list_range(
    conn: &mut SqliteConnection,
    start_bound: &str,
    end_bound: &str,
) -> Result<Vec<ScheduledBlock>, AppError> {
    let blocks = sqlx::query_as::<_, ScheduledBlock>(
        "SELECT * FROM scheduled_blocks
         WHERE start_at <= ?2 AND end_at >= ?1
         ORDER BY start_at, id",
    )
    .bind(start_bound)
    .bind(end_bound)
    .fetch_all(&mut *conn)
    .await?;
    Ok(blocks)
}

pub async fn update(
    conn: &mut SqliteConnection,
    id: i64,
    input: &ScheduledBlockInput,
    block_type: &str,
) -> Result<Option<ScheduledBlock>, AppError> {
    let block = sqlx::query_as::<_, ScheduledBlock>(
        "UPDATE scheduled_blocks
         SET title = ?, start_at = ?, end_at = ?, task_id = ?, block_type = ?
         WHERE id = ?
         RETURNING *",
    )
    .bind(&input.title)
    .bind(&input.start_at)
    .bind(&input.end_at)
    .bind(input.task_id)
    .bind(block_type)
    .bind(id)
    .fetch_optional(&mut *conn)
    .await?;
    Ok(block)
}

pub async fn set_status(
    conn: &mut SqliteConnection,
    id: i64,
    status: &str,
) -> Result<Option<ScheduledBlock>, AppError> {
    let block = sqlx::query_as::<_, ScheduledBlock>(
        "UPDATE scheduled_blocks SET status = ? WHERE id = ? RETURNING *",
    )
    .bind(status)
    .bind(id)
    .fetch_optional(&mut *conn)
    .await?;
    Ok(block)
}

pub async fn delete(conn: &mut SqliteConnection, id: i64) -> Result<bool, AppError> {
    let result = sqlx::query("DELETE FROM scheduled_blocks WHERE id = ?")
        .bind(id)
        .execute(&mut *conn)
        .await?;
    Ok(result.rows_affected() > 0)
}
