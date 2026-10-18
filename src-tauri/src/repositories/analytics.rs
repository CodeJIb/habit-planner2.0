use sqlx::SqliteConnection;

use crate::errors::AppError;
use crate::models::analytics::{DayCount, DayMinutes};

/// Net XP earned per local day, oldest first, as `(date, amount)`.
///
/// `created_at` is UTC, so days are grouped with `date(..., 'localtime')` to
/// match the "local day" the rest of the app works in.
pub async fn daily_xp(conn: &mut SqliteConnection) -> Result<Vec<(String, i64)>, AppError> {
    let rows = sqlx::query_as::<_, (String, i64)>(
        "SELECT date(created_at, 'localtime') AS day, COALESCE(SUM(amount), 0)
         FROM xp_transactions
         GROUP BY day
         ORDER BY day ASC",
    )
    .fetch_all(&mut *conn)
    .await?;
    Ok(rows)
}

/// Net XP from every local day strictly before `since` ("YYYY-MM-DD").
/// Used as the running total the visible window starts from.
pub async fn xp_before(conn: &mut SqliteConnection, since: &str) -> Result<i64, AppError> {
    let total: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(amount), 0) FROM xp_transactions
         WHERE date(created_at, 'localtime') < ?",
    )
    .bind(since)
    .fetch_one(&mut *conn)
    .await?;
    Ok(total)
}

/// Tasks completed per local day, from `since` ("YYYY-MM-DD") onward.
pub async fn task_completions(
    conn: &mut SqliteConnection,
    since: &str,
) -> Result<Vec<DayCount>, AppError> {
    let rows = sqlx::query_as::<_, DayCount>(
        "SELECT date(completed_at, 'localtime') AS date, COUNT(*) AS count
         FROM tasks
         WHERE status = 'done' AND completed_at IS NOT NULL
           AND date(completed_at, 'localtime') >= ?
         GROUP BY date
         ORDER BY date ASC",
    )
    .bind(since)
    .fetch_all(&mut *conn)
    .await?;
    Ok(rows)
}

/// Focused minutes per local day for completed sessions, from `since` onward.
/// `started_at` is written as a local wall-clock timestamp, so the day is its own prefix.
pub async fn focus_minutes(
    conn: &mut SqliteConnection,
    since: &str,
) -> Result<Vec<DayMinutes>, AppError> {
    let rows = sqlx::query_as::<_, DayMinutes>(
        "SELECT substr(started_at, 1, 10) AS date,
                SUM(COALESCE(actual_duration_seconds, 0)) / 60 AS minutes
         FROM focus_sessions
         WHERE status = 'completed' AND substr(started_at, 1, 10) >= ?
         GROUP BY date
         ORDER BY date ASC",
    )
    .bind(since)
    .fetch_all(&mut *conn)
    .await?;
    Ok(rows)
}

/// Every distinct local date with at least one habit check-in (the heat map source).
pub async fn checkin_dates(conn: &mut SqliteConnection) -> Result<Vec<String>, AppError> {
    let dates = sqlx::query_scalar::<_, String>(
        "SELECT DISTINCT local_date FROM habit_completions ORDER BY local_date ASC",
    )
    .fetch_all(&mut *conn)
    .await?;
    Ok(dates)
}

/// `(tasks_completed, focus_seconds, goals_completed)` across all time.
pub async fn totals(conn: &mut SqliteConnection) -> Result<(i64, i64, i64), AppError> {
    let row: (i64, i64, i64) = sqlx::query_as(
        "SELECT
            (SELECT COUNT(*) FROM tasks WHERE status = 'done'),
            (SELECT COALESCE(SUM(COALESCE(actual_duration_seconds, 0)), 0)
               FROM focus_sessions WHERE status = 'completed'),
            (SELECT COUNT(*) FROM goals WHERE status = 'completed')",
    )
    .fetch_one(&mut *conn)
    .await?;
    Ok(row)
}
