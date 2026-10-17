use sqlx::{FromRow, SqliteConnection};

use crate::errors::AppError;
use crate::models::quest::{Quest, QuestView};

/// All quest columns plus the joined progress value.
#[derive(FromRow)]
struct QuestWithProgress {
    id: i64,
    code: Option<String>,
    title: String,
    description: Option<String>,
    quest_type: String,
    metric: String,
    target_value: i64,
    xp_reward: i64,
    start_date: String,
    end_date: String,
    status: String,
    period_key: Option<String>,
    completed_at: Option<String>,
    xp_rewarded: bool,
    created_at: String,
    current_value: i64,
}

impl QuestWithProgress {
    fn into_view(self) -> QuestView {
        QuestView {
            completed: self.status == "completed",
            quest: Quest {
                id: self.id,
                code: self.code,
                title: self.title,
                description: self.description,
                quest_type: self.quest_type,
                metric: self.metric,
                target_value: self.target_value,
                xp_reward: self.xp_reward,
                start_date: self.start_date,
                end_date: self.end_date,
                status: self.status,
                period_key: self.period_key,
                completed_at: self.completed_at,
                xp_rewarded: self.xp_rewarded,
                created_at: self.created_at,
            },
            current_value: self.current_value,
        }
    }
}

/// Mark past, still-active quests as expired.
pub async fn expire_before(conn: &mut SqliteConnection, today: &str) -> Result<(), AppError> {
    sqlx::query("UPDATE quests SET status = 'expired' WHERE status = 'active' AND end_date < ?")
        .bind(today)
        .execute(&mut *conn)
        .await?;
    Ok(())
}

pub async fn find_by_code_period(
    conn: &mut SqliteConnection,
    code: &str,
    period_key: &str,
) -> Result<Option<i64>, AppError> {
    let id = sqlx::query_scalar::<_, i64>(
        "SELECT id FROM quests WHERE code = ? AND period_key = ?",
    )
    .bind(code)
    .bind(period_key)
    .fetch_optional(&mut *conn)
    .await?;
    Ok(id)
}

#[allow(clippy::too_many_arguments)]
pub async fn insert(
    conn: &mut SqliteConnection,
    code: &str,
    title: &str,
    description: &str,
    quest_type: &str,
    metric: &str,
    target_value: i64,
    xp_reward: i64,
    start_date: &str,
    end_date: &str,
    period_key: &str,
) -> Result<i64, AppError> {
    let id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO quests (code, title, description, quest_type, metric, target_value,
                             xp_reward, start_date, end_date, status, period_key)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 'active', ?)
         RETURNING id",
    )
    .bind(code)
    .bind(title)
    .bind(description)
    .bind(quest_type)
    .bind(metric)
    .bind(target_value)
    .bind(xp_reward)
    .bind(start_date)
    .bind(end_date)
    .bind(period_key)
    .fetch_one(&mut *conn)
    .await?;
    Ok(id)
}

pub async fn insert_progress(conn: &mut SqliteConnection, quest_id: i64) -> Result<(), AppError> {
    sqlx::query("INSERT INTO quest_progress (quest_id, current_value) VALUES (?, 0)")
        .bind(quest_id)
        .execute(&mut *conn)
        .await?;
    Ok(())
}

/// Active quests whose period covers `today`.
pub async fn active_for_date(
    conn: &mut SqliteConnection,
    today: &str,
) -> Result<Vec<Quest>, AppError> {
    let quests = sqlx::query_as::<_, Quest>(
        "SELECT * FROM quests
         WHERE status = 'active' AND period_key IS NOT NULL
           AND start_date <= ? AND end_date >= ?
         ORDER BY quest_type, id",
    )
    .bind(today)
    .bind(today)
    .fetch_all(&mut *conn)
    .await?;
    Ok(quests)
}

pub async fn set_progress(
    conn: &mut SqliteConnection,
    quest_id: i64,
    value: i64,
) -> Result<(), AppError> {
    sqlx::query("UPDATE quest_progress SET current_value = ? WHERE quest_id = ?")
        .bind(value)
        .bind(quest_id)
        .execute(&mut *conn)
        .await?;
    Ok(())
}

/// Mark a quest completed once, returning true only for the first transition.
pub async fn complete_once(conn: &mut SqliteConnection, quest_id: i64) -> Result<bool, AppError> {
    let result = sqlx::query(
        "UPDATE quests
         SET status = 'completed', completed_at = strftime('%Y-%m-%dT%H:%M:%SZ','now'), xp_rewarded = 1
         WHERE id = ? AND status = 'active' AND xp_rewarded = 0",
    )
    .bind(quest_id)
    .execute(&mut *conn)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Quests for the day/week containing `today`, with progress.
pub async fn views_for_date(
    conn: &mut SqliteConnection,
    today: &str,
) -> Result<Vec<QuestView>, AppError> {
    let rows = sqlx::query_as::<_, QuestWithProgress>(
        "SELECT q.id, q.code, q.title, q.description, q.quest_type, q.metric, q.target_value,
                q.xp_reward, q.start_date, q.end_date, q.status, q.period_key, q.completed_at,
                q.xp_rewarded, q.created_at, COALESCE(p.current_value, 0) AS current_value
         FROM quests q
         LEFT JOIN quest_progress p ON p.quest_id = q.id
         WHERE q.period_key IS NOT NULL AND q.start_date <= ?1 AND q.end_date >= ?1
         ORDER BY q.quest_type, q.id",
    )
    .bind(today)
    .fetch_all(&mut *conn)
    .await?;
    Ok(rows.into_iter().map(QuestWithProgress::into_view).collect())
}

/// Count tasks completed (by local completion date) within a date range.
pub async fn count_tasks_completed(
    conn: &mut SqliteConnection,
    start: &str,
    end: &str,
) -> Result<i64, AppError> {
    let count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM tasks
         WHERE status = 'done' AND completed_at IS NOT NULL
           AND date(completed_at, 'localtime') BETWEEN ? AND ?",
    )
    .bind(start)
    .bind(end)
    .fetch_one(&mut *conn)
    .await?;
    Ok(count)
}

/// Count habit check-ins within a date range (local_date is already local).
pub async fn count_habits_completed(
    conn: &mut SqliteConnection,
    start: &str,
    end: &str,
) -> Result<i64, AppError> {
    let count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM habit_completions WHERE local_date BETWEEN ? AND ?",
    )
    .bind(start)
    .bind(end)
    .fetch_one(&mut *conn)
    .await?;
    Ok(count)
}
