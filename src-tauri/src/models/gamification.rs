use serde::Serialize;

/// What the UI needs to draw the level / XP bar.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerProfileView {
    pub level: i64,
    pub total_xp: i64,
    /// XP earned since the start of the current level.
    pub xp_into_level: i64,
    /// Total XP span of the current level (so progress = xp_into_level / xp_for_next).
    pub xp_for_next: i64,
}

/// Result of an XP change, returned by any command that can award or revoke XP.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct XpOutcome {
    /// Actual change applied (negative when XP was revoked, 0 when nothing changed).
    pub xp_delta: i64,
    pub leveled_up: bool,
    pub profile: PlayerProfileView,
}
