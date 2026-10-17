use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use super::gamification::XpOutcome;

/// Goal row.
#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Goal {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub target_date: Option<String>,
    pub status: String,
    pub created_at: String,
    pub completed_at: Option<String>,
}

/// What the frontend sends to create or edit a goal (no id / timestamps).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoalInput {
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub target_date: Option<String>,
}

/// A milestone that belongs to a goal.
#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct GoalMilestone {
    pub id: i64,
    pub goal_id: i64,
    pub title: String,
    pub target_date: Option<String>,
    pub status: String,
    pub completed_at: Option<String>,
}

/// What the frontend sends to create a milestone.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MilestoneInput {
    pub title: String,
    #[serde(default)]
    pub target_date: Option<String>,
}

/// A goal plus its milestones, as shown in the UI.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GoalView {
    #[serde(flatten)]
    pub goal: Goal,
    pub milestones: Vec<GoalMilestone>,
}

/// Returned by status changes so the UI can update the goal and the XP bar together.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GoalStatusResult {
    pub goal: GoalView,
    pub xp: XpOutcome,
}
