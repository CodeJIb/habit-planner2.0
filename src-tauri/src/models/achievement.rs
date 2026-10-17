use serde::Serialize;
use sqlx::FromRow;

/// An achievement definition (seeded by a migration).
#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Achievement {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub xp_reward: i64,
}

/// An achievement plus whether this player has unlocked it.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AchievementView {
    #[serde(flatten)]
    pub achievement: Achievement,
    pub unlocked: bool,
    pub unlocked_at: Option<String>,
}
