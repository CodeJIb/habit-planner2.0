use sqlx::SqliteConnection;

use crate::errors::AppError;

/// A task that needs reminding about: just enough to build a notification.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DueTask {
    pub id: i64,
    pub title: String,
    pub due_at: String,
}

// Open = anything the user still has to do.
const OPEN: &str = "status IN ('todo', 'in_progress')";

/// Open tasks whose due moment has arrived and that have not been reminded yet,
/// oldest first. `now` is the user's local "YYYY-MM-DDTHH:MM".
pub async fn due_unreminded(
    conn: &mut SqliteConnection,
    now: &str,
    limit: i64,
) -> Result<Vec<DueTask>, AppError> {
    let sql = format!(
        "SELECT id, title, due_at FROM tasks
         WHERE {OPEN} AND due_at IS NOT NULL AND reminded_at IS NULL AND due_at <= ?
         ORDER BY due_at ASC, id ASC LIMIT ?"
    );
    let tasks = sqlx::query_as::<_, DueTask>(&sql)
        .bind(now)
        .bind(limit)
        .fetch_all(&mut *conn)
        .await?;
    Ok(tasks)
}

/// Mark tasks as reminded so an alert is never sent twice.
pub async fn mark_reminded(conn: &mut SqliteConnection, id: i64, at: &str) -> Result<(), AppError> {
    sqlx::query("UPDATE tasks SET reminded_at = ? WHERE id = ?")
        .bind(at)
        .bind(id)
        .execute(&mut *conn)
        .await?;
    Ok(())
}

/// A habit whose reminder time has arrived: just enough to build a notification.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DueHabit {
    pub id: i64,
    pub title: String,
}

/// Active habits whose reminder time has arrived and that have not been reminded
/// today, earliest first. `now_time` and `today` are the user's local "HH:MM" and
/// "YYYY-MM-DD".
pub async fn habits_due(
    conn: &mut SqliteConnection,
    now_time: &str,
    today: &str,
    limit: i64,
) -> Result<Vec<DueHabit>, AppError> {
    let habits = sqlx::query_as::<_, DueHabit>(
        "SELECT id, title FROM habits
         WHERE is_active = 1 AND reminder_time IS NOT NULL AND reminder_time <> ''
           AND reminder_time <= ? AND (reminded_on IS NULL OR reminded_on <> ?)
         ORDER BY reminder_time ASC, id ASC LIMIT ?",
    )
    .bind(now_time)
    .bind(today)
    .bind(limit)
    .fetch_all(&mut *conn)
    .await?;
    Ok(habits)
}

/// Mark a habit as reminded for the given day so it fires at most once per day.
pub async fn mark_habit_reminded(
    conn: &mut SqliteConnection,
    id: i64,
    today: &str,
) -> Result<(), AppError> {
    sqlx::query("UPDATE habits SET reminded_on = ? WHERE id = ?")
        .bind(today)
        .bind(id)
        .execute(&mut *conn)
        .await?;
    Ok(())
}

/// Open tasks due today or already overdue, for the daily digest.
pub async fn due_or_overdue(
    conn: &mut SqliteConnection,
    today: &str,
    limit: i64,
) -> Result<Vec<DueTask>, AppError> {
    let sql = format!(
        "SELECT id, title, due_at FROM tasks
         WHERE {OPEN} AND due_at IS NOT NULL AND substr(due_at, 1, 10) <= ?
         ORDER BY due_at ASC, id ASC LIMIT ?"
    );
    let tasks = sqlx::query_as::<_, DueTask>(&sql)
        .bind(today)
        .bind(limit)
        .fetch_all(&mut *conn)
        .await?;
    Ok(tasks)
}
