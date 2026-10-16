use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use super::gamification::XpOutcome;

/// A task as stored in the database.
#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: String,
    pub category_id: Option<i64>,
    pub due_at: Option<String>,
    pub scheduled_start_at: Option<String>,
    pub estimated_duration_minutes: Option<i64>,
    pub recurrence_rule: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub completed_at: Option<String>,
    pub archived_at: Option<String>,
}

fn default_priority() -> String {
    "medium".to_string()
}

/// What the frontend sends to create or edit a task (no id / timestamps).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskInput {
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default = "default_priority")]
    pub priority: String,
    #[serde(default)]
    pub category_id: Option<i64>,
    #[serde(default)]
    pub due_at: Option<String>,
    #[serde(default)]
    pub scheduled_start_at: Option<String>,
    #[serde(default)]
    pub estimated_duration_minutes: Option<i64>,
    #[serde(default)]
    pub recurrence_rule: Option<String>,
}

/// Returned by `set_task_status` so the UI can update the task and the XP bar in one go.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskStatusResult {
    pub task: Task,
    pub xp: XpOutcome,
}
