use sqlx::SqliteConnection;

use crate::errors::AppError;
use crate::models::achievement::AchievementView;
use crate::repositories::achievement as repo;
use crate::repositories::habit as habit_repo;
use crate::services::gamification;
use crate::services::habit::compute_streak;
use crate::services::quest::local_today;

/// Aggregate numbers achievements are judged against.
struct Stats {
    tasks_completed: i64,
    habit_completions: i64,
    best_streak: i64,
    level: i64,
}

async fn gather_stats(conn: &mut SqliteConnection) -> Result<Stats, AppError> {
    let tasks_completed =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM tasks WHERE status = 'done'")
            .fetch_one(&mut *conn)
            .await?;
    let habit_completions =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM habit_completions")
            .fetch_one(&mut *conn)
            .await?;
    let level = gamification::get_profile(conn).await?.level;

    // Best current streak across all active habits.
    let habit_ids = sqlx::query_scalar::<_, i64>("SELECT id FROM habits WHERE is_active = 1")
        .fetch_all(&mut *conn)
        .await?;
    let today = local_today();
    let mut best_streak = 0;
    for habit_id in habit_ids {
        let dates: Vec<_> = habit_repo::completion_dates(conn, habit_id)
            .await?
            .iter()
            .filter_map(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
            .collect();
        best_streak = best_streak.max(compute_streak(&dates, today));
    }

    Ok(Stats {
        tasks_completed,
        habit_completions,
        best_streak,
        level,
    })
}

/// Whether a player with these stats deserves an achievement.
fn condition_met(code: &str, stats: &Stats) -> bool {
    match code {
        "first_task" => stats.tasks_completed >= 1,
        "task_master_10" => stats.tasks_completed >= 10,
        "task_master_50" => stats.tasks_completed >= 50,
        "first_habit" => stats.habit_completions >= 1,
        "streak_7" => stats.best_streak >= 7,
        "level_5" => stats.level >= 5,
        _ => false,
    }
}

/// Unlock any newly-earned achievements, awarding XP once each.
pub async fn check(conn: &mut SqliteConnection) -> Result<(), AppError> {
    let stats = gather_stats(conn).await?;
    for view in repo::list_views(conn).await? {
        if view.unlocked {
            continue;
        }
        if condition_met(&view.achievement.code, &stats)
            && repo::unlock(conn, view.achievement.id).await?
        {
            let reason = format!("Achievement: {}", view.achievement.name);
            gamification::apply_xp(
                conn,
                view.achievement.xp_reward,
                "achievement",
                view.achievement.id,
                &reason,
            )
            .await?;
        }
    }
    Ok(())
}

/// All achievements with their unlock state, after catching up any pending unlocks.
pub async fn list(conn: &mut SqliteConnection) -> Result<Vec<AchievementView>, AppError> {
    check(conn).await?;
    repo::list_views(conn).await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stats(tasks: i64, habits: i64, streak: i64, level: i64) -> Stats {
        Stats {
            tasks_completed: tasks,
            habit_completions: habits,
            best_streak: streak,
            level,
        }
    }

    #[test]
    fn task_achievements_scale() {
        assert!(condition_met("first_task", &stats(1, 0, 0, 1)));
        assert!(!condition_met("task_master_10", &stats(9, 0, 0, 1)));
        assert!(condition_met("task_master_10", &stats(10, 0, 0, 1)));
        assert!(condition_met("task_master_50", &stats(50, 0, 0, 1)));
    }

    #[test]
    fn habit_and_level_achievements() {
        assert!(condition_met("first_habit", &stats(0, 1, 0, 1)));
        assert!(!condition_met("streak_7", &stats(0, 0, 6, 1)));
        assert!(condition_met("streak_7", &stats(0, 0, 7, 1)));
        assert!(condition_met("level_5", &stats(0, 0, 0, 5)));
        assert!(!condition_met("level_5", &stats(0, 0, 0, 4)));
    }

    #[test]
    fn unknown_code_never_unlocks() {
        assert!(!condition_met("nope", &stats(999, 999, 999, 99)));
    }
}
