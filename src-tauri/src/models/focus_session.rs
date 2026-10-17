use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::models::gamification::XpOutcome;

/// A logged focus session (one Pomodoro). Breaks are not sessions.
#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct FocusSession {
    pub id: i64,
    pub task_id: Option<i64>,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub planned_duration_minutes: i64,
    pub actual_duration_seconds: Option<i64>,
    pub status: String,
}

/// What the frontend sends to start a session.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FocusSessionInput {
    #[serde(default)]
    pub task_id: Option<i64>,
    pub planned_duration_minutes: i64,
}

/// A session plus the XP its completion earned (0 when nothing changed).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FocusResult {
    pub session: FocusSession,
    pub xp: XpOutcome,
}

/// Totals for the Focus screen header.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FocusStats {
    pub today_sessions: i64,
    pub today_minutes: i64,
    pub week_sessions: i64,
    pub week_minutes: i64,
}
