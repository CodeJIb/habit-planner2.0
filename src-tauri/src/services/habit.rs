use std::collections::HashSet;

use chrono::{Duration, NaiveDate};
use sqlx::{SqliteConnection, SqlitePool};

use crate::errors::AppError;
use crate::models::habit::{Habit, HabitInput, HabitToggleResult, HabitView};
use crate::repositories::habit as repo;
use crate::services::gamification;
use crate::services::{achievement, quest};

const FREQUENCIES: [&str; 3] = ["daily", "weekly", "custom"];
const HABIT_BASE_XP: i64 = 10;
const MAX_STREAK_BONUS_DAYS: i64 = 10;

fn validate(mut input: HabitInput) -> Result<HabitInput, AppError> {
    input.title = input.title.trim().to_string();
    if input.title.is_empty() {
        return Err(AppError::InvalidInput("Title is required".into()));
    }
    if input.title.chars().count() > 200 {
        return Err(AppError::InvalidInput("Title must be 200 characters or fewer".into()));
    }
    if !FREQUENCIES.contains(&input.frequency_type.as_str()) {
        return Err(AppError::InvalidInput(format!(
            "Unknown frequency '{}'",
            input.frequency_type
        )));
    }
    if let Some(count) = input.target_count {
        if count <= 0 {
            return Err(AppError::InvalidInput("Target count must be positive".into()));
        }
    }
    input.description = input
        .description
        .map(|d| d.trim().to_string())
        .filter(|d| !d.is_empty());
    Ok(input)
}

fn parse_date(value: &str) -> Result<NaiveDate, AppError> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| AppError::InvalidInput(format!("'{value}' is not a YYYY-MM-DD date")))
}

/// Number of consecutive days with a completion, counting back from `today`.
/// If today is not done yet the streak is still alive (it counts back from yesterday),
/// so a streak only breaks once a whole day has been missed.
pub fn compute_streak(dates: &[NaiveDate], today: NaiveDate) -> i64 {
    let done: HashSet<NaiveDate> = dates.iter().copied().collect();
    let mut cursor = if done.contains(&today) {
        today
    } else {
        today - Duration::days(1)
    };
    let mut streak = 0;
    while done.contains(&cursor) {
        streak += 1;
        cursor -= Duration::days(1);
    }
    streak
}

async fn load_dates(conn: &mut SqliteConnection, habit_id: i64) -> Result<Vec<NaiveDate>, AppError> {
    let raw = repo::completion_dates(conn, habit_id).await?;
    Ok(raw
        .iter()
        .filter_map(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
        .collect())
}

pub struct HabitService {
    pool: SqlitePool,
}

impl HabitService {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create_habit(&self, input: HabitInput) -> Result<Habit, AppError> {
        let input = validate(input)?;
        let mut conn = self.pool.acquire().await?;
        repo::insert(&mut conn, &input).await
    }

    /// Active habits with today's check-in state and current streak.
    /// `today` is the user's *local* date, sent by the frontend ("YYYY-MM-DD").
    pub async fn list_habits(&self, today: &str) -> Result<Vec<HabitView>, AppError> {
        let today = parse_date(today)?;
        let mut conn = self.pool.acquire().await?;
        let habits = repo::list_active(&mut conn).await?;

        let mut views = Vec::with_capacity(habits.len());
        for habit in habits {
            let dates = load_dates(&mut conn, habit.id).await?;
            views.push(HabitView {
                completed_today: dates.contains(&today),
                current_streak: compute_streak(&dates, today),
                habit,
            });
        }
        Ok(views)
    }

    pub async fn update_habit(&self, id: i64, input: HabitInput) -> Result<Habit, AppError> {
        let input = validate(input)?;
        let mut conn = self.pool.acquire().await?;
        repo::update(&mut conn, id, &input).await?.ok_or(AppError::NotFound)
    }

    pub async fn delete_habit(&self, id: i64) -> Result<(), AppError> {
        let mut conn = self.pool.acquire().await?;
        if repo::delete(&mut conn, id).await? {
            Ok(())
        } else {
            Err(AppError::NotFound)
        }
    }

    /// Check a habit in (or undo the check-in) for one local date.
    /// Check-in awards XP (base + a small streak bonus); undoing it takes that XP back.
    pub async fn toggle_completion(
        &self,
        habit_id: i64,
        local_date: &str,
    ) -> Result<HabitToggleResult, AppError> {
        let date = parse_date(local_date)?;
        let date_str = date.to_string(); // canonical "YYYY-MM-DD"

        let mut tx = self.pool.begin().await?;

        let habit = repo::find(&mut tx, habit_id).await?.ok_or(AppError::NotFound)?;

        let (completed, mut xp) = match repo::find_completion_id(&mut tx, habit_id, &date_str).await? {
            Some(completion_id) => {
                repo::delete_completion(&mut tx, completion_id).await?;
                let xp = gamification::revoke_xp(
                    &mut tx,
                    "habit_completion",
                    completion_id,
                    "Habit check-in undone",
                )
                .await?;
                (false, xp)
            }
            None => {
                let completion_id = repo::insert_completion(&mut tx, habit_id, &date_str).await?;
                let dates = load_dates(&mut tx, habit_id).await?;
                let streak = compute_streak(&dates, date);
                let amount = HABIT_BASE_XP + streak.min(MAX_STREAK_BONUS_DAYS) * 2;
                let reason = format!("Habit: {}", habit.title);
                let xp = gamification::apply_xp(&mut tx, amount, "habit_completion", completion_id, &reason)
                    .await?;
                (true, xp)
            }
        };

        let dates = load_dates(&mut tx, habit_id).await?;
        let current_streak = compute_streak(&dates, date);

        // A check-in may finish a quest or unlock an achievement; refresh both in
        // this transaction and pick up any XP they awarded.
        quest::recompute(&mut tx, true).await?;
        achievement::check(&mut tx).await?;
        xp.profile = gamification::get_profile(&mut tx).await?;

        tx.commit().await?;
        Ok(HabitToggleResult { completed, current_streak, xp })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    #[test]
    fn streak_counts_consecutive_days() {
        let dates = [d("2026-10-16"), d("2026-10-15"), d("2026-10-14")];
        assert_eq!(compute_streak(&dates, d("2026-10-16")), 3);
    }

    #[test]
    fn streak_survives_until_today_is_missed() {
        // Not done today yet, but done yesterday and the day before: still 2.
        let dates = [d("2026-10-15"), d("2026-10-14")];
        assert_eq!(compute_streak(&dates, d("2026-10-16")), 2);
    }

    #[test]
    fn streak_breaks_after_a_missed_day() {
        let dates = [d("2026-10-13"), d("2026-10-12")];
        assert_eq!(compute_streak(&dates, d("2026-10-16")), 0);
    }

    #[test]
    fn gap_ends_the_streak() {
        let dates = [d("2026-10-16"), d("2026-10-14")];
        assert_eq!(compute_streak(&dates, d("2026-10-16")), 1);
    }

    #[test]
    fn rejects_bad_dates() {
        assert!(parse_date("16/10/2026").is_err());
        assert!(parse_date("2026-10-16").is_ok());
    }
}
