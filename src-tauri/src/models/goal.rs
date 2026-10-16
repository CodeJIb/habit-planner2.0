use serde::Serialize;
use sqlx::FromRow;

/// Goal row (Phase 4 will add services/commands for it).
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
