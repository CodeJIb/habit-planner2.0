use chrono::{Datelike, Duration, Local, NaiveDate};
use sqlx::SqliteConnection;

use crate::errors::AppError;
use crate::models::quest::QuestView;
use crate::repositories::quest as repo;
use crate::services::gamification;

/// A quest definition that is materialised into a real quest row each period.
struct Template {
    code: &'static str,
    title: &'static str,
    description: &'static str,
    quest_type: &'static str,
    metric: &'static str,
    target_value: i64,
    xp_reward: i64,
}

/// The daily and weekly quests every player gets.
const TEMPLATES: [Template; 4] = [
    Template {
        code: "daily_tasks",
        title: "Clear 3 tasks",
        description: "Complete 3 tasks today.",
        quest_type: "daily",
        metric: "tasks_completed",
        target_value: 3,
        xp_reward: 30,
    },
    Template {
        code: "daily_habits",
        title: "Check in twice",
        description: "Log 2 habit check-ins today.",
        quest_type: "daily",
        metric: "habits_completed",
        target_value: 2,
        xp_reward: 20,
    },
    Template {
        code: "weekly_tasks",
        title: "Productive week",
        description: "Complete 12 tasks this week.",
        quest_type: "weekly",
        metric: "tasks_completed",
        target_value: 12,
        xp_reward: 120,
    },
    Template {
        code: "weekly_habits",
        title: "Consistency",
        description: "Log 10 habit check-ins this week.",
        quest_type: "weekly",
        metric: "habits_completed",
        target_value: 10,
        xp_reward: 80,
    },
];

/// The user's local date. Safe inside the desktop app where the backend runs on the same machine.
pub fn local_today() -> NaiveDate {
    Local::now().date_naive()
}

/// Start, end and stable key for the period a quest type falls in.
/// Weeks run Monday..Sunday, matching the habit week strip in the UI.
fn period_for(quest_type: &str, today: NaiveDate) -> (NaiveDate, NaiveDate, String) {
    if quest_type == "weekly" {
        let offset = today.weekday().num_days_from_monday() as i64;
        let start = today - Duration::days(offset);
        let end = start + Duration::days(6);
        (start, end, format!("weekly:{start}"))
    } else {
        (today, today, format!("daily:{today}"))
    }
}

/// Create the current period's quest rows if they do not exist yet.
pub async fn ensure(conn: &mut SqliteConnection) -> Result<(), AppError> {
    let today = local_today();
    repo::expire_before(conn, &today.to_string()).await?;

    for template in TEMPLATES.iter() {
        let (start, end, key) = period_for(template.quest_type, today);
        if repo::find_by_code_period(conn, template.code, &key).await?.is_none() {
            let id = repo::insert(
                conn,
                template.code,
                template.title,
                template.description,
                template.quest_type,
                template.metric,
                template.target_value,
                template.xp_reward,
                &start.to_string(),
                &end.to_string(),
                &key,
            )
            .await?;
            repo::insert_progress(conn, id).await?;
        }
    }
    Ok(())
}

/// Recompute progress for the current period's quests from source data, and
/// (when `award` is set) complete and reward any quest that reached its target.
/// Deriving progress from the tasks/habits themselves keeps it tamper-proof:
/// toggling the same task never counts twice.
pub async fn recompute(conn: &mut SqliteConnection, award: bool) -> Result<(), AppError> {
    ensure(conn).await?;
    let today = local_today().to_string();

    for quest in repo::active_for_date(conn, &today).await? {
        let value = match quest.metric.as_str() {
            "habits_completed" => {
                repo::count_habits_completed(conn, &quest.start_date, &quest.end_date).await?
            }
            _ => repo::count_tasks_completed(conn, &quest.start_date, &quest.end_date).await?,
        };
        let value = value.min(quest.target_value);
        repo::set_progress(conn, quest.id, value).await?;

        if award && value >= quest.target_value && repo::complete_once(conn, quest.id).await? {
            let reason = format!("Quest completed: {}", quest.title);
            gamification::apply_xp(conn, quest.xp_reward, "quest", quest.id, &reason).await?;
        }
    }
    Ok(())
}

/// Quests for the day/week containing today, refreshed before returning.
pub async fn list(conn: &mut SqliteConnection) -> Result<Vec<QuestView>, AppError> {
    recompute(conn, true).await?;
    let today = local_today().to_string();
    repo::views_for_date(conn, &today).await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    #[test]
    fn daily_period_is_the_day_itself() {
        let today = d("2026-10-16");
        let (start, end, key) = period_for("daily", today);
        assert_eq!(start, today);
        assert_eq!(end, today);
        assert_eq!(key, "daily:2026-10-16");
    }

    #[test]
    fn weekly_period_runs_monday_to_sunday() {
        // 2026-10-16 is a Friday.
        let (start, end, key) = period_for("weekly", d("2026-10-16"));
        assert_eq!(start, d("2026-10-12")); // Monday
        assert_eq!(end, d("2026-10-18")); // Sunday
        assert_eq!(key, "weekly:2026-10-12");
    }

    #[test]
    fn weekly_period_from_monday_and_sunday() {
        assert_eq!(period_for("weekly", d("2026-10-12")).0, d("2026-10-12"));
        assert_eq!(period_for("weekly", d("2026-10-18")).0, d("2026-10-12"));
    }
}
