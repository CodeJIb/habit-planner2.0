use sqlx::SqliteConnection;

use crate::errors::AppError;
use crate::models::goal::{Goal, GoalInput, GoalMilestone, MilestoneInput};

// SQL fragment for "now" in the same ISO format the schema uses for defaults.
const NOW: &str = "strftime('%Y-%m-%dT%H:%M:%SZ','now')";

pub async fn insert(conn: &mut SqliteConnection, input: &GoalInput) -> Result<Goal, AppError> {
    let goal = sqlx::query_as::<_, Goal>(
        "INSERT INTO goals (title, description, target_date)
         VALUES (?, ?, ?)
         RETURNING *",
    )
    .bind(&input.title)
    .bind(&input.description)
    .bind(&input.target_date)
    .fetch_one(&mut *conn)
    .await?;
    Ok(goal)
}

pub async fn find(conn: &mut SqliteConnection, id: i64) -> Result<Option<Goal>, AppError> {
    let goal = sqlx::query_as::<_, Goal>("SELECT * FROM goals WHERE id = ?")
        .bind(id)
        .fetch_optional(&mut *conn)
        .await?;
    Ok(goal)
}

pub async fn list(conn: &mut SqliteConnection, status: Option<&str>) -> Result<Vec<Goal>, AppError> {
    let goals = match status {
        Some(s) => {
            sqlx::query_as::<_, Goal>(
                "SELECT * FROM goals WHERE status = ? ORDER BY created_at DESC, id DESC",
            )
            .bind(s)
            .fetch_all(&mut *conn)
            .await?
        }
        None => {
            sqlx::query_as::<_, Goal>("SELECT * FROM goals ORDER BY created_at DESC, id DESC")
                .fetch_all(&mut *conn)
                .await?
        }
    };
    Ok(goals)
}

pub async fn update(
    conn: &mut SqliteConnection,
    id: i64,
    input: &GoalInput,
) -> Result<Option<Goal>, AppError> {
    let goal = sqlx::query_as::<_, Goal>(
        "UPDATE goals SET title = ?, description = ?, target_date = ? WHERE id = ? RETURNING *",
    )
    .bind(&input.title)
    .bind(&input.description)
    .bind(&input.target_date)
    .bind(id)
    .fetch_optional(&mut *conn)
    .await?;
    Ok(goal)
}

/// Change status and keep `completed_at` consistent with it.
pub async fn set_status(
    conn: &mut SqliteConnection,
    id: i64,
    status: &str,
) -> Result<Option<Goal>, AppError> {
    let sql = format!(
        "UPDATE goals
         SET status = ?1,
             completed_at = CASE WHEN ?1 = 'completed' THEN {NOW} ELSE NULL END
         WHERE id = ?2
         RETURNING *"
    );
    let goal = sqlx::query_as::<_, Goal>(&sql)
        .bind(status)
        .bind(id)
        .fetch_optional(&mut *conn)
        .await?;
    Ok(goal)
}

pub async fn delete(conn: &mut SqliteConnection, id: i64) -> Result<bool, AppError> {
    let result = sqlx::query("DELETE FROM goals WHERE id = ?")
        .bind(id)
        .execute(&mut *conn)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn insert_milestone(
    conn: &mut SqliteConnection,
    goal_id: i64,
    input: &MilestoneInput,
) -> Result<GoalMilestone, AppError> {
    let milestone = sqlx::query_as::<_, GoalMilestone>(
        "INSERT INTO goal_milestones (goal_id, title, target_date)
         VALUES (?, ?, ?)
         RETURNING *",
    )
    .bind(goal_id)
    .bind(&input.title)
    .bind(&input.target_date)
    .fetch_one(&mut *conn)
    .await?;
    Ok(milestone)
}

pub async fn list_milestones(
    conn: &mut SqliteConnection,
    goal_id: i64,
) -> Result<Vec<GoalMilestone>, AppError> {
    let milestones = sqlx::query_as::<_, GoalMilestone>(
        "SELECT * FROM goal_milestones WHERE goal_id = ? ORDER BY id",
    )
    .bind(goal_id)
    .fetch_all(&mut *conn)
    .await?;
    Ok(milestones)
}

pub async fn find_milestone(
    conn: &mut SqliteConnection,
    id: i64,
) -> Result<Option<GoalMilestone>, AppError> {
    let milestone = sqlx::query_as::<_, GoalMilestone>("SELECT * FROM goal_milestones WHERE id = ?")
        .bind(id)
        .fetch_optional(&mut *conn)
        .await?;
    Ok(milestone)
}

/// Change a milestone's status and keep `completed_at` consistent with it.
pub async fn set_milestone_status(
    conn: &mut SqliteConnection,
    id: i64,
    status: &str,
) -> Result<Option<GoalMilestone>, AppError> {
    let sql = format!(
        "UPDATE goal_milestones
         SET status = ?1,
             completed_at = CASE WHEN ?1 = 'completed' THEN {NOW} ELSE NULL END
         WHERE id = ?2
         RETURNING *"
    );
    let milestone = sqlx::query_as::<_, GoalMilestone>(&sql)
        .bind(status)
        .bind(id)
        .fetch_optional(&mut *conn)
        .await?;
    Ok(milestone)
}

pub async fn delete_milestone(conn: &mut SqliteConnection, id: i64) -> Result<bool, AppError> {
    let result = sqlx::query("DELETE FROM goal_milestones WHERE id = ?")
        .bind(id)
        .execute(&mut *conn)
        .await?;
    Ok(result.rows_affected() > 0)
}
