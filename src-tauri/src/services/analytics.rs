use chrono::{Duration, Local, NaiveDate};
use sqlx::SqlitePool;

use crate::errors::AppError;
use crate::models::analytics::{AnalyticsSummary, DayCount, DayMinutes, XpPoint};
use crate::repositories::analytics as repo;
use crate::services::{gamification, habit};

/// The task and focus bar charts default to two weeks when no window is given.
const DEFAULT_CHART_DAYS: i64 = 14;
/// Ceiling on a "last N days" request, so a silly value cannot ask for a decade of rows.
const MAX_WINDOW_DAYS: i64 = 3660;

fn today() -> NaiveDate {
    Local::now().date_naive()
}

/// The first local day of a "last N days" window, inclusive.
fn window_start(days: i64) -> NaiveDate {
    today() - Duration::days(days - 1)
}

/// Clamp a "last N days" request. `None` falls back to `default`, and both being
/// `None` means "no window" (the whole history).
fn normalize_days(days: Option<i64>, default: Option<i64>) -> Result<Option<i64>, AppError> {
    match days.or(default) {
        None => Ok(None),
        Some(days) if days < 1 => Err(AppError::InvalidInput("Days must be at least 1".into())),
        Some(days) => Ok(Some(days.min(MAX_WINDOW_DAYS))),
    }
}

pub struct AnalyticsService {
    pool: SqlitePool,
}

impl AnalyticsService {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Cumulative XP per local day, oldest first. With no window the whole ledger is
    /// charted; with one, the series still starts from the XP earned before it, so the
    /// line and the level thresholds stay meaningful.
    pub async fn xp_over_time(&self, days: Option<i64>) -> Result<Vec<XpPoint>, AppError> {
        let days = normalize_days(days, None)?;
        let mut conn = self.pool.acquire().await?;
        let daily = repo::daily_xp(&mut conn).await?;

        let (since, mut cumulative) = match days {
            Some(days) => {
                let since = window_start(days).to_string();
                let before = repo::xp_before(&mut conn, &since).await?;
                (Some(since), before)
            }
            None => (None, 0),
        };

        let mut points = Vec::new();
        for (date, amount) in daily {
            if let Some(since) = since.as_deref() {
                // "YYYY-MM-DD" compares chronologically as a string.
                if date.as_str() < since {
                    continue;
                }
            }
            cumulative += amount;
            points.push(XpPoint {
                date,
                amount,
                cumulative,
            });
        }
        Ok(points)
    }

    /// Tasks completed per local day, defaulting to the last two weeks.
    pub async fn task_completions(&self, days: Option<i64>) -> Result<Vec<DayCount>, AppError> {
        let days = normalize_days(days, Some(DEFAULT_CHART_DAYS))?.unwrap_or(DEFAULT_CHART_DAYS);
        let since = window_start(days).to_string();
        let mut conn = self.pool.acquire().await?;
        repo::task_completions(&mut conn, &since).await
    }

    /// Focused minutes per local day for completed sessions, defaulting to the last two weeks.
    pub async fn focus_minutes(&self, days: Option<i64>) -> Result<Vec<DayMinutes>, AppError> {
        let days = normalize_days(days, Some(DEFAULT_CHART_DAYS))?.unwrap_or(DEFAULT_CHART_DAYS);
        let since = window_start(days).to_string();
        let mut conn = self.pool.acquire().await?;
        repo::focus_minutes(&mut conn, &since).await
    }

    /// Every local date holding at least one habit check-in (the heat map source).
    pub async fn habit_checkins(&self) -> Result<Vec<String>, AppError> {
        let mut conn = self.pool.acquire().await?;
        repo::checkin_dates(&mut conn).await
    }

    /// Totals behind the summary cards. `longest_streak` is the best run of consecutive
    /// check-in days any active habit has ever had, not just the streak still running.
    pub async fn summary(&self) -> Result<AnalyticsSummary, AppError> {
        let mut conn = self.pool.acquire().await?;
        let profile = gamification::get_profile(&mut conn).await?;
        let (tasks_completed, focus_seconds, goals_completed) = repo::totals(&mut conn).await?;

        let mut longest_streak = 0;
        for habit in crate::repositories::habit::list_active(&mut conn).await? {
            let dates = habit::load_dates(&mut conn, habit.id).await?;
            longest_streak = longest_streak.max(habit::longest_run(&dates));
        }

        Ok(AnalyticsSummary {
            total_xp: profile.total_xp,
            level: profile.level,
            longest_streak,
            tasks_completed,
            focus_minutes: focus_seconds / 60,
            goals_completed,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_start_is_inclusive_of_today() {
        // A 14-day window ends today and starts 13 days back.
        let start = window_start(14);
        assert_eq!(start, today() - Duration::days(13));
        assert_eq!(window_start(1), today());
    }

    #[test]
    fn days_are_clamped_and_defaulted() {
        assert_eq!(normalize_days(None, None).unwrap(), None, "no default means all time");
        assert_eq!(normalize_days(None, Some(14)).unwrap(), Some(14));
        assert_eq!(normalize_days(Some(30), Some(14)).unwrap(), Some(30));
        assert_eq!(
            normalize_days(Some(99_999), None).unwrap(),
            Some(MAX_WINDOW_DAYS),
            "a runaway window is capped"
        );
        assert!(normalize_days(Some(0), None).is_err());
        assert!(normalize_days(Some(-3), Some(14)).is_err());
    }
}
