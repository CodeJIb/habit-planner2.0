use sqlx::SqliteConnection;

use crate::errors::AppError;
use crate::models::focus_session::FocusSession;

pub async fn insert(
    conn: &mut SqliteConnection,
    task_id: Option<i64>,
    planned_duration_minutes: i64,
    started_at: &str,
) -> Result<FocusSession, AppError> {
    let session = sqlx::query_as::<_, FocusSession>(
        "INSERT INTO focus_sessions (task_id, started_at, planned_duration_minutes)
         VALUES (?, ?, ?)
         RETURNING *",
    )
    .bind(task_id)
    .bind(started_at)
    .bind(planned_duration_minutes)
    .fetch_one(&mut *conn)
    .await?;
    Ok(session)
}

pub async fn find(conn: &mut SqliteConnection, id: i64) -> Result<Option<FocusSession>, AppError> {
    let session = sqlx::query_as::<_, FocusSession>("SELECT * FROM focus_sessions WHERE id = ?")
        .bind(id)
        .fetch_optional(&mut *conn)
        .await?;
    Ok(session)
}

/// Most recent sessions first, optionally only those linked to `task_id`.
pub async fn list(
    conn: &mut SqliteConnection,
    task_id: Option<i64>,
    limit: i64,
) -> Result<Vec<FocusSession>, AppError> {
    let sessions = if let Some(task_id) = task_id {
        sqlx::query_as::<_, FocusSession>(
            "SELECT * FROM focus_sessions
             WHERE task_id = ?
             ORDER BY started_at DESC, id DESC
             LIMIT ?",
        )
        .bind(task_id)
        .bind(limit)
        .fetch_all(&mut *conn)
        .await?
    } else {
        sqlx::query_as::<_, FocusSession>(
            "SELECT * FROM focus_sessions ORDER BY started_at DESC, id DESC LIMIT ?",
        )
        .bind(limit)
        .fetch_all(&mut *conn)
        .await?
    };
    Ok(sessions)
}

pub async fn complete(
    conn: &mut SqliteConnection,
    id: i64,
    ended_at: &str,
    actual_duration_seconds: i64,
) -> Result<Option<FocusSession>, AppError> {
    let session = sqlx::query_as::<_, FocusSession>(
        "UPDATE focus_sessions
         SET status = 'completed', ended_at = ?, actual_duration_seconds = ?
         WHERE id = ?
         RETURNING *",
    )
    .bind(ended_at)
    .bind(actual_duration_seconds)
    .bind(id)
    .fetch_optional(&mut *conn)
    .await?;
    Ok(session)
}

pub async fn cancel(
    conn: &mut SqliteConnection,
    id: i64,
    ended_at: &str,
) -> Result<Option<FocusSession>, AppError> {
    let session = sqlx::query_as::<_, FocusSession>(
        "UPDATE focus_sessions
         SET status = 'cancelled', ended_at = ?
         WHERE id = ?
         RETURNING *",
    )
    .bind(ended_at)
    .bind(id)
    .fetch_optional(&mut *conn)
    .await?;
    Ok(session)
}

pub async fn delete(conn: &mut SqliteConnection, id: i64) -> Result<bool, AppError> {
    let result = sqlx::query("DELETE FROM focus_sessions WHERE id = ?")
        .bind(id)
        .execute(&mut *conn)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// (today_sessions, today_seconds, week_sessions, week_seconds) for completed sessions.
/// `today` and `week_start` are "YYYY-MM-DD"; dates are compared on the day the session started.
pub async fn stats(
    conn: &mut SqliteConnection,
    today: &str,
    week_start: &str,
) -> Result<(i64, i64, i64, i64), AppError> {
    let row: (i64, i64, i64, i64) = sqlx::query_as(
        "SELECT
            COALESCE(SUM(CASE WHEN substr(started_at, 1, 10) = ?1 THEN 1 ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN substr(started_at, 1, 10) = ?1 THEN COALESCE(actual_duration_seconds, 0) ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN substr(started_at, 1, 10) >= ?2 THEN 1 ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN substr(started_at, 1, 10) >= ?2 THEN COALESCE(actual_duration_seconds, 0) ELSE 0 END), 0)
         FROM focus_sessions
         WHERE status = 'completed'",
    )
    .bind(today)
    .bind(week_start)
    .fetch_one(&mut *conn)
    .await?;
    Ok(row)
}
