use sqlx::SqlitePool;

use crate::errors::AppError;
use crate::models::goal::{Goal, GoalInput, GoalStatusResult, GoalView, MilestoneInput};
use crate::repositories::goal as repo;
use crate::services::gamification;

const STATUSES: [&str; 3] = ["active", "completed", "archived"];

/// XP awarded for finishing a goal.
const GOAL_COMPLETION_XP: i64 = 100;

fn validate_goal(mut input: GoalInput) -> Result<GoalInput, AppError> {
    input.title = input.title.trim().to_string();
    if input.title.is_empty() {
        return Err(AppError::InvalidInput("Title is required".into()));
    }
    if input.title.chars().count() > 200 {
        return Err(AppError::InvalidInput("Title must be 200 characters or fewer".into()));
    }
    input.description = input
        .description
        .map(|d| d.trim().to_string())
        .filter(|d| !d.is_empty());
    Ok(input)
}

fn validate_milestone(mut input: MilestoneInput) -> Result<MilestoneInput, AppError> {
    input.title = input.title.trim().to_string();
    if input.title.is_empty() {
        return Err(AppError::InvalidInput("Title is required".into()));
    }
    if input.title.chars().count() > 200 {
        return Err(AppError::InvalidInput("Title must be 200 characters or fewer".into()));
    }
    Ok(input)
}

fn check_status(status: &str) -> Result<(), AppError> {
    if STATUSES.contains(&status) {
        Ok(())
    } else {
        Err(AppError::InvalidInput(format!("Unknown status '{status}'")))
    }
}

/// Attach a goal's milestones to make a view for the UI.
async fn to_view(
    conn: &mut sqlx::SqliteConnection,
    goal: Goal,
) -> Result<GoalView, AppError> {
    let milestones = repo::list_milestones(conn, goal.id).await?;
    Ok(GoalView { goal, milestones })
}

pub struct GoalService {
    pool: SqlitePool,
}

impl GoalService {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create_goal(&self, input: GoalInput) -> Result<GoalView, AppError> {
        let input = validate_goal(input)?;
        let mut conn = self.pool.acquire().await?;
        let goal = repo::insert(&mut conn, &input).await?;
        to_view(&mut conn, goal).await
    }

    /// Goals with their milestones. `status` filters when given.
    pub async fn list_goals(&self, status: Option<String>) -> Result<Vec<GoalView>, AppError> {
        if let Some(s) = &status {
            check_status(s)?;
        }
        let mut conn = self.pool.acquire().await?;
        let goals = repo::list(&mut conn, status.as_deref()).await?;
        let mut views = Vec::with_capacity(goals.len());
        for goal in goals {
            views.push(to_view(&mut conn, goal).await?);
        }
        Ok(views)
    }

    pub async fn update_goal(&self, id: i64, input: GoalInput) -> Result<GoalView, AppError> {
        let input = validate_goal(input)?;
        let mut conn = self.pool.acquire().await?;
        let goal = repo::update(&mut conn, id, &input).await?.ok_or(AppError::NotFound)?;
        to_view(&mut conn, goal).await
    }

    /// Change a goal's status. Finishing a goal awards XP; reopening it takes that XP back.
    /// Status, XP and the ledger update in one transaction so they can never disagree.
    pub async fn set_status(&self, id: i64, status: String) -> Result<GoalStatusResult, AppError> {
        check_status(&status)?;

        let mut tx = self.pool.begin().await?;

        let current = repo::find(&mut tx, id).await?.ok_or(AppError::NotFound)?;

        let xp = if status == "completed" && current.status != "completed" {
            let reason = format!("Completed goal: {}", current.title);
            gamification::apply_xp(&mut tx, GOAL_COMPLETION_XP, "goal", id, &reason).await?
        } else if status != "completed" && current.status == "completed" {
            gamification::revoke_xp(&mut tx, "goal", id, "Goal reopened").await?
        } else {
            gamification::current_outcome(&mut tx).await?
        };

        let goal = repo::set_status(&mut tx, id, &status)
            .await?
            .ok_or(AppError::NotFound)?;
        let view = to_view(&mut tx, goal).await?;

        tx.commit().await?;
        Ok(GoalStatusResult { goal: view, xp })
    }

    pub async fn delete_goal(&self, id: i64) -> Result<(), AppError> {
        let mut conn = self.pool.acquire().await?;
        if repo::delete(&mut conn, id).await? {
            Ok(())
        } else {
            Err(AppError::NotFound)
        }
    }

    pub async fn add_milestone(&self, goal_id: i64, input: MilestoneInput) -> Result<GoalView, AppError> {
        let input = validate_milestone(input)?;
        let mut conn = self.pool.acquire().await?;
        let goal = repo::find(&mut conn, goal_id).await?.ok_or(AppError::NotFound)?;
        repo::insert_milestone(&mut conn, goal_id, &input).await?;
        to_view(&mut conn, goal).await
    }

    /// Toggle a milestone between completed and active, returning its goal.
    pub async fn set_milestone_status(
        &self,
        id: i64,
        status: String,
    ) -> Result<GoalView, AppError> {
        check_status(&status)?;
        let mut conn = self.pool.acquire().await?;
        let milestone = repo::find_milestone(&mut conn, id)
            .await?
            .ok_or(AppError::NotFound)?;
        repo::set_milestone_status(&mut conn, id, &status).await?;
        let goal = repo::find(&mut conn, milestone.goal_id)
            .await?
            .ok_or(AppError::NotFound)?;
        to_view(&mut conn, goal).await
    }

    pub async fn delete_milestone(&self, id: i64) -> Result<GoalView, AppError> {
        let mut conn = self.pool.acquire().await?;
        let milestone = repo::find_milestone(&mut conn, id)
            .await?
            .ok_or(AppError::NotFound)?;
        repo::delete_milestone(&mut conn, id).await?;
        let goal = repo::find(&mut conn, milestone.goal_id)
            .await?
            .ok_or(AppError::NotFound)?;
        to_view(&mut conn, goal).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn goal_input(title: &str) -> GoalInput {
        GoalInput {
            title: title.to_string(),
            description: Some("   ".to_string()),
            target_date: None,
        }
    }

    fn milestone_input(title: &str) -> MilestoneInput {
        MilestoneInput {
            title: title.to_string(),
            target_date: None,
        }
    }

    #[test]
    fn rejects_empty_goal_title() {
        assert!(validate_goal(goal_input("   ")).is_err());
    }

    #[test]
    fn trims_goal_title_and_blank_description() {
        let cleaned = validate_goal(goal_input("  Run a marathon  ")).unwrap();
        assert_eq!(cleaned.title, "Run a marathon");
        assert!(cleaned.description.is_none());
    }

    #[test]
    fn rejects_empty_milestone_title() {
        assert!(validate_milestone(milestone_input("")).is_err());
    }

    #[test]
    fn accepts_known_statuses_only() {
        assert!(check_status("active").is_ok());
        assert!(check_status("completed").is_ok());
        assert!(check_status("archived").is_ok());
        assert!(check_status("done").is_err());
    }
}
