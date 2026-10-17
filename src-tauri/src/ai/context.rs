//! Phase 5: build the context (tasks, habits, XP) that is sent to the model.
//!
//! The context is deliberately small: it targets a low-end phone on slow mobile
//! data and a free-tier provider with a tight per-minute token budget. So the
//! most relevant rows come first (overdue and due-today tasks), long titles are
//! truncated, and every list has a hard cap with a "…and N more" tail so the
//! model still knows data exists beyond what it can see.

use chrono::NaiveDate;
use sqlx::SqlitePool;

use crate::errors::AppError;
use crate::repositories::task as task_repo;
use crate::services::focus_session::FocusSessionService;
use crate::services::goal::GoalService;
use crate::services::habit::HabitService;
use crate::services::{gamification, quest};

const MAX_TASKS: usize = 12;
const MAX_HABITS: usize = 12;
const MAX_GOALS: usize = 6;
const MAX_QUESTS: usize = 8;
/// Long titles are cut here so one verbose row cannot eat the whole budget.
const TITLE_MAX: usize = 60;

/// A compact, plain-text snapshot of the player's data. The model never sees
/// the database directly — only what this function chooses to include.
pub async fn build(pool: &SqlitePool, today: &str) -> Result<String, AppError> {
    let mut conn = pool.acquire().await?;

    let profile = gamification::get_profile(&mut conn).await?;
    let tasks = task_repo::list(&mut conn, None).await?;
    let quests = quest::list(&mut conn).await?;
    drop(conn);

    let habits = HabitService::new(pool.clone()).list_habits(today).await?;
    let goals = GoalService::new(pool.clone()).list_goals(None).await?;
    let focus = FocusSessionService::new(pool.clone()).stats().await?;

    let mut out = String::new();

    out.push_str(&format!("TODAY: {}", human_date(today)));
    out.push_str(&format!(
        "\n\nPLAYER\nLevel {} · {} total XP · {} of {} XP toward the next level.",
        profile.level, profile.total_xp, profile.xp_into_level, profile.xp_for_next
    ));

    // ---- tasks ------------------------------------------------------------
    // Only unfinished work is useful to the model. Within that, overdue and
    // due-today tasks matter most, so they are listed first.
    let mut open: Vec<_> = tasks
        .iter()
        .filter(|t| t.status == "todo" || t.status == "in_progress")
        .collect();
    open.sort_by_key(|t| due_bucket(t.due_at.as_deref(), today));

    let overdue = open
        .iter()
        .filter(|t| due_bucket(t.due_at.as_deref(), today) == 0)
        .count();
    let due_today = open
        .iter()
        .filter(|t| due_bucket(t.due_at.as_deref(), today) == 1)
        .count();
    let undated = open.iter().filter(|t| t.due_at.is_none()).count();
    let done_today = tasks
        .iter()
        .filter(|t| t.status == "done" && t.completed_at.as_deref().is_some_and(|c| c.starts_with(today)))
        .count();

    out.push_str(&format!(
        "\n\nTASKS\nOpen: {} ({} overdue, {} due today, {} undated) · Completed today: {}.",
        open.len(),
        overdue,
        due_today,
        undated,
        done_today
    ));
    for task in open.iter().take(MAX_TASKS) {
        let due = task
            .due_at
            .as_deref()
            .map(|d| format!(", due {d}"))
            .unwrap_or_default();
        out.push_str(&format!(
            "\n- [{}] {} (id {}, {}{})",
            task.priority,
            truncate(&task.title, TITLE_MAX),
            task.id,
            task.status,
            due
        ));
    }
    if open.len() > MAX_TASKS {
        out.push_str(&format!("\n…and {} more open tasks.", open.len() - MAX_TASKS));
    }
    if open.is_empty() {
        out.push_str("\n- none");
    }

    // ---- habits -----------------------------------------------------------
    // `list_habits` already returns active habits only.
    out.push_str("\n\nHABITS (today)");
    for habit in habits.iter().take(MAX_HABITS) {
        out.push_str(&format!(
            "\n- {} (id {}, {}): streak {} day(s), {}",
            truncate(&habit.habit.title, TITLE_MAX),
            habit.habit.id,
            habit.habit.frequency_type,
            habit.current_streak,
            if habit.completed_today {
                "done today"
            } else {
                "not checked in yet"
            }
        ));
    }
    if habits.len() > MAX_HABITS {
        out.push_str(&format!(
            "\n…and {} more habits.",
            habits.len() - MAX_HABITS
        ));
    }
    if habits.is_empty() {
        out.push_str("\n- none");
    }

    // ---- quests -----------------------------------------------------------
    let current: Vec<_> = quests
        .iter()
        .filter(|q| q.quest.status != "expired")
        .collect();
    if !current.is_empty() {
        out.push_str("\n\nQUESTS");
        for quest in current.iter().take(MAX_QUESTS) {
            out.push_str(&format!(
                "\n- [{}] {} {}/{} (+{} XP){}",
                quest.quest.quest_type,
                truncate(&quest.quest.title, TITLE_MAX),
                quest.current_value,
                quest.quest.target_value,
                quest.quest.xp_reward,
                if quest.completed { " — done" } else { "" }
            ));
        }
        if current.len() > MAX_QUESTS {
            out.push_str(&format!("\n…and {} more quests.", current.len() - MAX_QUESTS));
        }
    }

    // ---- goals ------------------------------------------------------------
    let active: Vec<_> = goals.iter().filter(|g| g.goal.status == "active").collect();
    if !active.is_empty() {
        out.push_str("\n\nGOALS (active)");
        for goal in active.iter().take(MAX_GOALS) {
            let done = goal
                .milestones
                .iter()
                .filter(|m| m.status == "completed")
                .count();
            let due = goal
                .goal
                .target_date
                .as_deref()
                .map(|d| format!(", target {d}"))
                .unwrap_or_default();
            out.push_str(&format!(
                "\n- {} (id {}{}): {}/{} milestones done",
                truncate(&goal.goal.title, TITLE_MAX),
                goal.goal.id,
                due,
                done,
                goal.milestones.len()
            ));
        }
        if active.len() > MAX_GOALS {
            out.push_str(&format!(
                "\n…and {} more goals.",
                active.len() - MAX_GOALS
            ));
        }
    }

    // ---- focus ------------------------------------------------------------
    out.push_str(&format!(
        "\n\nFOCUS\nToday: {} min in {} session(s) · This week: {} min in {} session(s).",
        focus.today_minutes, focus.today_sessions, focus.week_minutes, focus.week_sessions
    ));

    Ok(out)
}

/// Sort key for open tasks: overdue first, then due today, then upcoming, then
/// undated. ISO dates compare correctly as strings.
fn due_bucket(due_at: Option<&str>, today: &str) -> u8 {
    match due_at {
        Some(due) if due < today => 0,
        Some(due) if due.starts_with(today) => 1,
        Some(_) => 2,
        None => 3,
    }
}

/// Cut `text` to at most `max` characters (not bytes), adding an ellipsis.
fn truncate(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    let mut out: String = text.chars().take(max.saturating_sub(1)).collect();
    out.push('…');
    out
}

/// "2026-10-16" -> "Friday, October 16, 2026 (local date 2026-10-16)".
fn human_date(today: &str) -> String {
    match NaiveDate::parse_from_str(today, "%Y-%m-%d") {
        Ok(date) => format!("{} (local date {today})", date.format("%A, %B %-d, %Y")),
        Err(_) => today.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn human_date_formats_a_real_date() {
        assert_eq!(
            human_date("2026-10-16"),
            "Friday, October 16, 2026 (local date 2026-10-16)"
        );
    }

    #[test]
    fn human_date_falls_back_to_the_raw_string() {
        assert_eq!(human_date("not-a-date"), "not-a-date");
    }

    #[test]
    fn overdue_sorts_before_due_today_before_upcoming_and_undated() {
        assert_eq!(due_bucket(Some("2026-10-15"), "2026-10-16"), 0);
        assert_eq!(due_bucket(Some("2026-10-16"), "2026-10-16"), 1);
        assert_eq!(due_bucket(Some("2026-10-16T09:00"), "2026-10-16"), 1);
        assert_eq!(due_bucket(Some("2026-10-20"), "2026-10-16"), 2);
        assert_eq!(due_bucket(None, "2026-10-16"), 3);
    }

    #[test]
    fn truncate_cuts_long_text_on_char_boundaries() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("abcdefghij", 5), "abcd…");
        // Multi-byte characters must not panic or split.
        assert_eq!(truncate("héllo wörld", 6), "héllo…");
    }
}
