use chrono::{NaiveDate, NaiveDateTime};
use sqlx::SqlitePool;

use crate::errors::AppError;
use crate::models::scheduled_block::{ScheduledBlock, ScheduledBlockInput};
use crate::repositories::scheduled_block as repo;

const STATUSES: [&str; 4] = ["scheduled", "running", "done", "cancelled"];
const BLOCK_TYPES: [&str; 3] = ["task", "break", "other"];
const DEFAULT_BLOCK_TYPE: &str = "task";

fn parse_date(s: &str) -> Result<NaiveDate, AppError> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .map_err(|_| AppError::InvalidInput(format!("'{s}' is not a YYYY-MM-DD date")))
}

/// Blocks are stored as local "YYYY-MM-DDTHH:MM"; anything else is rejected early
/// so a malformed time cannot silently break the range query.
fn parse_datetime(s: &str) -> Result<(), AppError> {
    NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M")
        .map(|_| ())
        .map_err(|_| AppError::InvalidInput(format!("'{s}' is not a YYYY-MM-DDTHH:MM time")))
}

/// Trim the title, check the times, and resolve the block type.
fn validate(mut input: ScheduledBlockInput) -> Result<(ScheduledBlockInput, String), AppError> {
    input.title = input.title.trim().to_string();
    if input.title.is_empty() {
        return Err(AppError::InvalidInput("Title is required".into()));
    }
    if input.title.chars().count() > 200 {
        return Err(AppError::InvalidInput("Title must be 200 characters or fewer".into()));
    }

    parse_datetime(&input.start_at)?;
    parse_datetime(&input.end_at)?;
    if input.end_at <= input.start_at {
        return Err(AppError::InvalidInput("End time must be after the start time".into()));
    }

    let block_type = match input.block_type.take() {
        Some(t) => {
            if !BLOCK_TYPES.contains(&t.as_str()) {
                return Err(AppError::InvalidInput(format!("Unknown block type '{t}'")));
            }
            t
        }
        None => DEFAULT_BLOCK_TYPE.to_string(),
    };

    Ok((input, block_type))
}

fn check_status(status: &str) -> Result<(), AppError> {
    if STATUSES.contains(&status) {
        Ok(())
    } else {
        Err(AppError::InvalidInput(format!("Unknown status '{status}'")))
    }
}

pub struct ScheduledBlockService {
    pool: SqlitePool,
}

impl ScheduledBlockService {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// All blocks, or only those overlapping the date range when both bounds are given.
    pub async fn list(
        &self,
        start_date: Option<String>,
        end_date: Option<String>,
    ) -> Result<Vec<ScheduledBlock>, AppError> {
        let mut conn = self.pool.acquire().await?;
        match (start_date, end_date) {
            (None, None) => repo::list_all(&mut conn).await,
            (Some(start), Some(end)) => {
                let start = parse_date(&start)?;
                let end = parse_date(&end)?;
                if start > end {
                    return Err(AppError::InvalidInput(
                        "Start date must not be after the end date".into(),
                    ));
                }
                repo::list_range(
                    &mut conn,
                    &format!("{start}T00:00"),
                    &format!("{end}T23:59"),
                )
                .await
            }
            _ => Err(AppError::InvalidInput(
                "Provide both a start and an end date, or neither".into(),
            )),
        }
    }

    pub async fn create(&self, input: ScheduledBlockInput) -> Result<ScheduledBlock, AppError> {
        let (input, block_type) = validate(input)?;
        let mut conn = self.pool.acquire().await?;
        repo::insert(&mut conn, &input, &block_type).await
    }

    pub async fn update(
        &self,
        id: i64,
        input: ScheduledBlockInput,
    ) -> Result<ScheduledBlock, AppError> {
        let (input, block_type) = validate(input)?;
        let mut conn = self.pool.acquire().await?;
        repo::update(&mut conn, id, &input, &block_type)
            .await?
            .ok_or(AppError::NotFound)
    }

    pub async fn set_status(&self, id: i64, status: String) -> Result<ScheduledBlock, AppError> {
        check_status(&status)?;
        let mut conn = self.pool.acquire().await?;
        repo::set_status(&mut conn, id, &status)
            .await?
            .ok_or(AppError::NotFound)
    }

    pub async fn delete(&self, id: i64) -> Result<(), AppError> {
        let mut conn = self.pool.acquire().await?;
        if repo::delete(&mut conn, id).await? {
            Ok(())
        } else {
            Err(AppError::NotFound)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(start: &str, end: &str) -> ScheduledBlockInput {
        ScheduledBlockInput {
            title: "  Deep work  ".into(),
            start_at: start.into(),
            end_at: end.into(),
            task_id: None,
            block_type: None,
        }
    }

    #[test]
    fn trims_title_and_defaults_block_type() {
        let (cleaned, block_type) = validate(input("2026-10-16T09:00", "2026-10-16T10:00")).unwrap();
        assert_eq!(cleaned.title, "Deep work");
        assert_eq!(block_type, "task");
    }

    #[test]
    fn rejects_end_before_start() {
        assert!(validate(input("2026-10-16T10:00", "2026-10-16T09:00")).is_err());
    }

    #[test]
    fn rejects_equal_start_and_end() {
        assert!(validate(input("2026-10-16T10:00", "2026-10-16T10:00")).is_err());
    }

    #[test]
    fn rejects_bad_time_and_unknown_type() {
        assert!(validate(input("2026/10/16 09:00", "2026-10-16T10:00")).is_err());

        let mut bad = input("2026-10-16T09:00", "2026-10-16T10:00");
        bad.block_type = Some("nap".into());
        assert!(validate(bad).is_err());
    }

    #[test]
    fn accepts_valid_block_type() {
        let mut ok = input("2026-10-16T09:00", "2026-10-16T10:00");
        ok.block_type = Some("break".into());
        let (_, block_type) = validate(ok).unwrap();
        assert_eq!(block_type, "break");
    }

    #[test]
    fn rejects_bad_date_bounds() {
        assert!(parse_date("16/10/2026").is_err());
        assert!(parse_date("2026-10-16").is_ok());
        assert!(check_status("running").is_ok());
        assert!(check_status("paused").is_err());
    }
}
