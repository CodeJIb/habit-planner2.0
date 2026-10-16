use sqlx::SqliteConnection;

use crate::errors::AppError;
use crate::models::habit::{Habit, HabitInput};

pub async fn insert(conn: &mut SqliteConnection, input: &HabitInput) -> Result<Habit, AppError> {
    let habit = sqlx::query_as::<_, Habit>(
        "INSERT INTO habits (title, description, category_id, frequency_type, frequency_config,
                             target_count, reminder_time)
         VALUES (?, ?, ?, ?, ?, ?, ?)
         RETURNING *",
    )
    .bind(&input.title)
    .bind(&input.description)
    .bind(input.category_id)
    .bind(&input.frequency_type)
    .bind(&input.frequency_config)
    .bind(input.target_count)
    .bind(&input.reminder_time)
    .fetch_one(&mut *conn)
    .await?;
    Ok(habit)
}

pub async fn find(conn: &mut SqliteConnection, id: i64) -> Result<Option<Habit>, AppError> {
    let habit = sqlx::query_as::<_, Habit>("SELECT * FROM habits WHERE id = ?")
        .bind(id)
        .fetch_optional(&mut *conn)
        .await?;
    Ok(habit)
}

pub async fn list_active(conn: &mut SqliteConnection) -> Result<Vec<Habit>, AppError> {
    let habits = sqlx::query_as::<_, Habit>(
        "SELECT * FROM habits WHERE is_active = 1 ORDER BY created_at, id",
    )
    .fetch_all(&mut *conn)
    .await?;
    Ok(habits)
}

pub async fn update(
    conn: &mut SqliteConnection,
    id: i64,
    input: &HabitInput,
) -> Result<Option<Habit>, AppError> {
    let habit = sqlx::query_as::<_, Habit>(
        "UPDATE habits
         SET title = ?, description = ?, category_id = ?, frequency_type = ?,
             frequency_config = ?, target_count = ?, reminder_time = ?
         WHERE id = ?
         RETURNING *",
    )
    .bind(&input.title)
    .bind(&input.description)
    .bind(input.category_id)
    .bind(&input.frequency_type)
    .bind(&input.frequency_config)
    .bind(input.target_count)
    .bind(&input.reminder_time)
    .bind(id)
    .fetch_optional(&mut *conn)
    .await?;
    Ok(habit)
}

pub async fn delete(conn: &mut SqliteConnection, id: i64) -> Result<bool, AppError> {
    let result = sqlx::query("DELETE FROM habits WHERE id = ?")
        .bind(id)
        .execute(&mut *conn)
        .await?;
    Ok(result.rows_affected() > 0)
}

// ---- completions -------------------------------------------------------------

pub async fn find_completion_id(
    conn: &mut SqliteConnection,
    habit_id: i64,
    local_date: &str,
) -> Result<Option<i64>, AppError> {
    let id = sqlx::query_scalar::<_, i64>(
        "SELECT id FROM habit_completions WHERE habit_id = ? AND local_date = ?",
    )
    .bind(habit_id)
    .bind(local_date)
    .fetch_optional(&mut *conn)
    .await?;
    Ok(id)
}

pub async fn insert_completion(
    conn: &mut SqliteConnection,
    habit_id: i64,
    local_date: &str,
) -> Result<i64, AppError> {
    let result = sqlx::query("INSERT INTO habit_completions (habit_id, local_date) VALUES (?, ?)")
        .bind(habit_id)
        .bind(local_date)
        .execute(&mut *conn)
        .await?;
    Ok(result.last_insert_rowid())
}

pub async fn delete_completion(conn: &mut SqliteConnection, id: i64) -> Result<(), AppError> {
    sqlx::query("DELETE FROM habit_completions WHERE id = ?")
        .bind(id)
        .execute(&mut *conn)
        .await?;
    Ok(())
}

/// All completed dates ("YYYY-MM-DD") for a habit, newest first.
pub async fn completion_dates(
    conn: &mut SqliteConnection,
    habit_id: i64,
) -> Result<Vec<String>, AppError> {
    let dates = sqlx::query_scalar::<_, String>(
        "SELECT local_date FROM habit_completions WHERE habit_id = ? ORDER BY local_date DESC",
    )
    .bind(habit_id)
    .fetch_all(&mut *conn)
    .await?;
    Ok(dates)
}
