use sqlx::{FromRow, SqliteConnection};

use crate::errors::AppError;
use crate::models::achievement::AchievementView;

#[derive(FromRow)]
struct AchievementRow {
    id: i64,
    code: String,
    name: String,
    description: Option<String>,
    icon: Option<String>,
    xp_reward: i64,
    unlocked_at: Option<String>,
}

/// Every achievement definition, with the player's unlock state.
pub async fn list_views(conn: &mut SqliteConnection) -> Result<Vec<AchievementView>, AppError> {
    let rows = sqlx::query_as::<_, AchievementRow>(
        "SELECT a.id, a.code, a.name, a.description, a.icon, a.xp_reward, ua.unlocked_at
         FROM achievements a
         LEFT JOIN user_achievements ua ON ua.achievement_id = a.id
         ORDER BY a.id",
    )
    .fetch_all(&mut *conn)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| AchievementView {
            unlocked: r.unlocked_at.is_some(),
            unlocked_at: r.unlocked_at,
            achievement: crate::models::achievement::Achievement {
                id: r.id,
                code: r.code,
                name: r.name,
                description: r.description,
                icon: r.icon,
                xp_reward: r.xp_reward,
            },
        })
        .collect())
}

/// Unlock an achievement once, returning true only when newly inserted.
pub async fn unlock(conn: &mut SqliteConnection, achievement_id: i64) -> Result<bool, AppError> {
    let result = sqlx::query("INSERT OR IGNORE INTO user_achievements (achievement_id) VALUES (?)")
        .bind(achievement_id)
        .execute(&mut *conn)
        .await?;
    Ok(result.rows_affected() > 0)
}
