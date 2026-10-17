use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// A time block on the calendar. It may be linked to a task (`task_id`) or stand alone.
#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ScheduledBlock {
    pub id: i64,
    pub task_id: Option<i64>,
    pub title: String,
    pub start_at: String,
    pub end_at: String,
    pub block_type: String,
    pub status: String,
    pub created_at: String,
}

/// What the frontend sends to create or edit a block.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduledBlockInput {
    pub title: String,
    /// Local datetime "YYYY-MM-DDTHH:MM" (no timezone; the desktop app and the
    /// database run on the same machine, so local time is unambiguous here).
    pub start_at: String,
    pub end_at: String,
    #[serde(default)]
    pub task_id: Option<i64>,
    #[serde(default)]
    pub block_type: Option<String>,
}
