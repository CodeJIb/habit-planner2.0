use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use super::gamification::XpOutcome;

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Habit {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub category_id: Option<i64>,
    pub frequency_type: String,
    pub frequency_config: Option<String>,
    pub target_count: Option<i64>,
    pub reminder_time: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub archived_at: Option<String>,
}

fn default_frequency() -> String {
    "daily".to_string()
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HabitInput {
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub category_id: Option<i64>,
    #[serde(default = "default_frequency")]
    pub frequency_type: String,
    #[serde(default)]
    pub frequency_config: Option<String>,
    #[serde(default)]
    pub target_count: Option<i64>,
    #[serde(default)]
    pub reminder_time: Option<String>,
}

/// A habit plus the data the daily checklist needs.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HabitView {
    #[serde(flatten)]
    pub habit: Habit,
    pub completed_today: bool,
    pub current_streak: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HabitToggleResult {
    pub completed: bool,
    pub current_streak: i64,
    pub xp: XpOutcome,
}
