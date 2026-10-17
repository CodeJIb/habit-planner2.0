use serde::Serialize;
use sqlx::FromRow;

/// A quest row. Quests are generated from templates for the current day/week.
#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Quest {
    pub id: i64,
    pub code: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub quest_type: String,
    pub metric: String,
    pub target_value: i64,
    pub xp_reward: i64,
    pub start_date: String,
    pub end_date: String,
    pub status: String,
    pub period_key: Option<String>,
    pub completed_at: Option<String>,
    pub xp_rewarded: bool,
    pub created_at: String,
}

/// A quest plus its progress, as shown in the UI.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestView {
    #[serde(flatten)]
    pub quest: Quest,
    pub current_value: i64,
    pub completed: bool,
}
