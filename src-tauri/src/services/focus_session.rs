use chrono::{Datelike, Duration, Local, NaiveDate};
use sqlx::SqlitePool;

use crate::errors::AppError;
use crate::models::focus_session::{FocusResult, FocusSession, FocusSessionInput, FocusStats};
use crate::repositories::focus_session as repo;
use crate::services::{gamification, quest};

/// XP earned per focused minute (capped at the planned duration, see `focus_xp`).
const FOCUS_XP_PER_MINUTE: i64 = 1;
/// Sanity ceiling so a runaway client cannot log an absurd session.
const MAX_PLANNED_MINUTES: i64 = 600;

/// Local wall-clock timestamp "YYYY-MM-DDTHH:MM:SS" (same machine as the DB).
fn local_now() -> String {
    Local::now().format("%Y-%m-%dT%H:%M:%S").to_string()
}

/// Monday of the week containing `today`, matching the habit week strip.
fn week_start(today: NaiveDate) -> NaiveDate {
    let offset = today.weekday().num_days_from_monday() as i64;
    today - Duration::days(offset)
}

fn validate_planned(minutes: i64) -> Result<(), AppError> {
    if minutes <= 0 {
        return Err(AppError::InvalidInput("Planned duration must be positive".into()));
    }
    if minutes > MAX_PLANNED_MINUTES {
        return Err(AppError::InvalidInput(format!(
            "Planned duration must be {MAX_PLANNED_MINUTES} minutes or fewer"
        )));
    }
    Ok(())
}

/// XP for a completed session: one per whole focused minute, never more than planned.
/// Capping stops a session that over-ran (or a spoofed duration) from inflating XP.
fn focus_xp(actual_seconds: i64, planned_minutes: i64) -> i64 {
    let minutes = (actual_seconds / 60).max(0);
    minutes.min(planned_minutes) * FOCUS_XP_PER_MINUTE
}

pub struct FocusSessionService {
    pool: SqlitePool,
}

impl FocusSessionService {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn start(&self, input: FocusSessionInput) -> Result<FocusSession, AppError> {
        validate_planned(input.planned_duration_minutes)?;
        let mut conn = self.pool.acquire().await?;
        repo::insert(
            &mut conn,
            input.task_id,
            input.planned_duration_minutes,
            &local_now(),
        )
        .await
    }

    /// Recent sessions, optionally filtered to one task. `limit` is clamped to 1..=500.
    pub async fn list(
        &self,
        task_id: Option<i64>,
        limit: Option<i64>,
    ) -> Result<Vec<FocusSession>, AppError> {
        let limit = limit.unwrap_or(50).clamp(1, 500);
        let mut conn = self.pool.acquire().await?;
        repo::list(&mut conn, task_id, limit).await
    }

    /// Finish a session and award XP for the focused time. Idempotent: completing an
    /// already-completed session never awards XP twice.
    pub async fn complete(
        &self,
        id: i64,
        actual_duration_seconds: i64,
    ) -> Result<FocusResult, AppError> {
        if actual_duration_seconds < 0 {
            return Err(AppError::InvalidInput(
                "Actual duration cannot be negative".into(),
            ));
        }

        let mut tx = self.pool.begin().await?;
        let current = repo::find(&mut tx, id).await?.ok_or(AppError::NotFound)?;
        if current.status == "cancelled" {
            return Err(AppError::InvalidInput("A cancelled session cannot be completed".into()));
        }

        let xp = if current.status != "completed" {
            let amount = focus_xp(actual_duration_seconds, current.planned_duration_minutes);
            if amount > 0 {
                let reason = format!("Focused {} min", amount);
                gamification::apply_xp(&mut tx, amount, "focus", id, &reason).await?
            } else {
                gamification::current_outcome(&mut tx).await?
            }
        } else {
            gamification::current_outcome(&mut tx).await?
        };

        let session = repo::complete(&mut tx, id, &local_now(), actual_duration_seconds)
            .await?
            .ok_or(AppError::NotFound)?;
        tx.commit().await?;
        Ok(FocusResult { session, xp })
    }

    pub async fn cancel(&self, id: i64) -> Result<FocusResult, AppError> {
        let mut tx = self.pool.begin().await?;
        let current = repo::find(&mut tx, id).await?.ok_or(AppError::NotFound)?;
        if current.status == "completed" {
            return Err(AppError::InvalidInput("A completed session cannot be cancelled".into()));
        }

        let session = repo::cancel(&mut tx, id, &local_now()).await?.ok_or(AppError::NotFound)?;
        let xp = gamification::current_outcome(&mut tx).await?;
        tx.commit().await?;
        Ok(FocusResult { session, xp })
    }

    /// Deleting a session also takes back any XP it earned.
    pub async fn delete(&self, id: i64) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await?;
        if repo::find(&mut tx, id).await?.is_none() {
            return Err(AppError::NotFound);
        }
        gamification::revoke_xp(&mut tx, "focus", id, "Focus session deleted").await?;
        repo::delete(&mut tx, id).await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn stats(&self) -> Result<FocusStats, AppError> {
        let today = quest::local_today();
        let week = week_start(today);
        let mut conn = self.pool.acquire().await?;
        let (today_sessions, today_seconds, week_sessions, week_seconds) =
            repo::stats(&mut conn, &today.to_string(), &week.to_string()).await?;
        Ok(FocusStats {
            today_sessions,
            today_minutes: today_seconds / 60,
            week_sessions,
            week_minutes: week_seconds / 60,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_positive_planned_duration() {
        assert!(validate_planned(0).is_err());
        assert!(validate_planned(-5).is_err());
        assert!(validate_planned(25).is_ok());
    }

    #[test]
    fn rejects_absurd_planned_duration() {
        assert!(validate_planned(MAX_PLANNED_MINUTES).is_ok());
        assert!(validate_planned(MAX_PLANNED_MINUTES + 1).is_err());
    }

    #[test]
    fn xp_is_one_per_focused_minute() {
        assert_eq!(focus_xp(0, 25), 0);
        assert_eq!(focus_xp(59, 25), 0);
        assert_eq!(focus_xp(60, 25), 1);
        assert_eq!(focus_xp(25 * 60, 25), 25);
    }

    #[test]
    fn xp_is_capped_at_the_planned_duration() {
        assert_eq!(focus_xp(60 * 60, 25), 25);
        assert_eq!(focus_xp(10 * 60, 25), 10);
    }

    #[test]
    fn week_start_is_monday() {
        // 2026-10-16 is a Friday.
        let friday = NaiveDate::from_ymd_opt(2026, 10, 16).unwrap();
        assert_eq!(week_start(friday).to_string(), "2026-10-12");
        let monday = NaiveDate::from_ymd_opt(2026, 10, 12).unwrap();
        assert_eq!(week_start(monday).to_string(), "2026-10-12");
        let sunday = NaiveDate::from_ymd_opt(2026, 10, 18).unwrap();
        assert_eq!(week_start(sunday).to_string(), "2026-10-12");
    }
}
