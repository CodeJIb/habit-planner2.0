use sqlx::SqlitePool;

use crate::errors::AppError;
use crate::models::task::{Task, TaskInput, TaskStatusResult};
use crate::repositories::task as repo;
use crate::services::gamification;

const PRIORITIES: [&str; 4] = ["low", "medium", "high", "urgent"];
const STATUSES: [&str; 4] = ["todo", "in_progress", "done", "archived"];

/// XP earned for finishing a task, by priority.
fn xp_for_priority(priority: &str) -> i64 {
    match priority {
        "low" => 10,
        "high" => 35,
        "urgent" => 50,
        _ => 20, // medium
    }
}

fn validate(mut input: TaskInput) -> Result<TaskInput, AppError> {
    input.title = input.title.trim().to_string();
    if input.title.is_empty() {
        return Err(AppError::InvalidInput("Title is required".into()));
    }
    if input.title.chars().count() > 200 {
        return Err(AppError::InvalidInput("Title must be 200 characters or fewer".into()));
    }
    if !PRIORITIES.contains(&input.priority.as_str()) {
        return Err(AppError::InvalidInput(format!("Unknown priority '{}'", input.priority)));
    }
    if let Some(minutes) = input.estimated_duration_minutes {
        if minutes <= 0 {
            return Err(AppError::InvalidInput("Estimated duration must be positive".into()));
        }
    }
    input.description = input
        .description
        .map(|d| d.trim().to_string())
        .filter(|d| !d.is_empty());
    Ok(input)
}

pub struct TaskService {
    pool: SqlitePool,
}

impl TaskService {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create_task(&self, input: TaskInput) -> Result<Task, AppError> {
        let input = validate(input)?;
        let mut conn = self.pool.acquire().await?;
        repo::insert(&mut conn, &input).await
    }

    pub async fn get_task(&self, id: i64) -> Result<Task, AppError> {
        let mut conn = self.pool.acquire().await?;
        repo::find(&mut conn, id).await?.ok_or(AppError::NotFound)
    }

    pub async fn list_tasks(&self, status: Option<String>) -> Result<Vec<Task>, AppError> {
        if let Some(s) = &status {
            if !STATUSES.contains(&s.as_str()) {
                return Err(AppError::InvalidInput(format!("Unknown status '{s}'")));
            }
        }
        let mut conn = self.pool.acquire().await?;
        repo::list(&mut conn, status.as_deref()).await
    }

    pub async fn update_task(&self, id: i64, input: TaskInput) -> Result<Task, AppError> {
        let input = validate(input)?;
        let mut conn = self.pool.acquire().await?;
        repo::update(&mut conn, id, &input).await?.ok_or(AppError::NotFound)
    }

    /// Change a task's status. Moving to "done" awards XP; moving away from "done" takes it back.
    /// Everything happens in one transaction, so XP and status can never disagree.
    pub async fn set_status(&self, id: i64, status: String) -> Result<TaskStatusResult, AppError> {
        if !STATUSES.contains(&status.as_str()) {
            return Err(AppError::InvalidInput(format!("Unknown status '{status}'")));
        }

        let mut tx = self.pool.begin().await?;

        let current = repo::find(&mut tx, id).await?.ok_or(AppError::NotFound)?;

        let xp = if status == "done" && current.status != "done" {
            let reason = format!("Completed task: {}", current.title);
            gamification::apply_xp(&mut tx, xp_for_priority(&current.priority), "task", id, &reason)
                .await?
        } else if status != "done" && current.status == "done" {
            gamification::revoke_xp(&mut tx, "task", id, "Task reopened").await?
        } else {
            gamification::current_outcome(&mut tx).await?
        };

        let task = repo::set_status(&mut tx, id, &status)
            .await?
            .ok_or(AppError::NotFound)?;

        tx.commit().await?;
        Ok(TaskStatusResult { task, xp })
    }

    pub async fn delete_task(&self, id: i64) -> Result<(), AppError> {
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

    fn input(title: &str, priority: &str) -> TaskInput {
        TaskInput {
            title: title.to_string(),
            description: Some("  ".to_string()),
            priority: priority.to_string(),
            category_id: None,
            due_at: None,
            scheduled_start_at: None,
            estimated_duration_minutes: None,
            recurrence_rule: None,
        }
    }

    #[test]
    fn rejects_empty_title() {
        assert!(validate(input("   ", "medium")).is_err());
    }

    #[test]
    fn rejects_unknown_priority() {
        assert!(validate(input("Write report", "someday")).is_err());
    }

    #[test]
    fn trims_title_and_blank_description() {
        let cleaned = validate(input("  Write report  ", "high")).unwrap();
        assert_eq!(cleaned.title, "Write report");
        assert!(cleaned.description.is_none());
    }

    #[test]
    fn xp_scales_with_priority() {
        assert!(xp_for_priority("urgent") > xp_for_priority("high"));
        assert!(xp_for_priority("high") > xp_for_priority("medium"));
        assert!(xp_for_priority("medium") > xp_for_priority("low"));
    }
}
