use serde::Serialize;
use sqlx::FromRow;

/// One point on the XP-over-time chart: what was earned that day and the running total after it.
#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct XpPoint {
    pub date: String,
    pub amount: i64,
    pub cumulative: i64,
}

/// A count for one local date (e.g. tasks completed that day).
#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct DayCount {
    pub date: String,
    pub count: i64,
}

/// Minutes for one local date (e.g. focused time that day).
#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct DayMinutes {
    pub date: String,
    pub minutes: i64,
}

/// The values behind the Analytics summary cards.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsSummary {
    pub total_xp: i64,
    pub level: i64,
    pub longest_streak: i64,
    pub tasks_completed: i64,
    pub focus_minutes: i64,
    pub goals_completed: i64,
}
