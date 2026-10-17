use sqlx::SqliteConnection;

use crate::errors::AppError;
use crate::models::gamification::{PlayerProfileView, XpOutcome};

const NOW: &str = "strftime('%Y-%m-%dT%H:%M:%SZ','now')";

/// Total XP required to *reach* `level`.
/// Level 1 = 0 XP, level 2 = 100, level 3 = 400, level 4 = 900, ... (100 * (level-1)^2)
pub fn xp_for_level(level: i64) -> i64 {
    let n = (level - 1).max(0);
    100 * n * n
}

/// The level a player with `xp` total XP is on.
pub fn level_for_xp(xp: i64) -> i64 {
    let mut level = 1;
    while xp >= xp_for_level(level + 1) {
        level += 1;
    }
    level
}

fn build_view(level: i64, total_xp: i64) -> PlayerProfileView {
    let start = xp_for_level(level);
    let next = xp_for_level(level + 1);
    PlayerProfileView {
        level,
        total_xp,
        xp_into_level: total_xp - start,
        xp_for_next: next - start,
    }
}

async fn read_profile(conn: &mut SqliteConnection) -> Result<(i64, i64), AppError> {
    let row: (i64, i64) =
        sqlx::query_as("SELECT current_level, total_xp FROM player_profile WHERE id = 1")
            .fetch_one(&mut *conn)
            .await?;
    Ok(row)
}

pub async fn get_profile(conn: &mut SqliteConnection) -> Result<PlayerProfileView, AppError> {
    let (level, total) = read_profile(conn).await?;
    Ok(build_view(level, total))
}

/// An outcome meaning "nothing changed" (still carries the current profile).
pub async fn current_outcome(conn: &mut SqliteConnection) -> Result<XpOutcome, AppError> {
    Ok(XpOutcome {
        xp_delta: 0,
        leveled_up: false,
        profile: get_profile(conn).await?,
    })
}

/// Add (or, if negative, remove) XP: writes a ledger row and updates the profile.
/// Call this inside the same transaction as the action that earned the XP.
pub async fn apply_xp(
    conn: &mut SqliteConnection,
    amount: i64,
    source_type: &str,
    source_id: i64,
    reason: &str,
) -> Result<XpOutcome, AppError> {
    if amount == 0 {
        return current_outcome(conn).await;
    }

    let (old_level, old_total) = read_profile(conn).await?;

    sqlx::query(
        "INSERT INTO xp_transactions (amount, source_type, source_id, reason) VALUES (?, ?, ?, ?)",
    )
    .bind(amount)
    .bind(source_type)
    .bind(source_id)
    .bind(reason)
    .execute(&mut *conn)
    .await?;

    let new_total = (old_total + amount).max(0);
    let new_level = level_for_xp(new_total);

    let sql = format!(
        "UPDATE player_profile SET total_xp = ?, current_level = ?, updated_at = {NOW} WHERE id = 1"
    );
    sqlx::query(&sql)
        .bind(new_total)
        .bind(new_level)
        .execute(&mut *conn)
        .await?;

    Ok(XpOutcome {
        xp_delta: new_total - old_total,
        leveled_up: new_level > old_level,
        profile: build_view(new_level, new_total),
    })
}

/// Undo whatever XP a source (a task, a habit check-in) currently has on the ledger.
pub async fn revoke_xp(
    conn: &mut SqliteConnection,
    source_type: &str,
    source_id: i64,
    reason: &str,
) -> Result<XpOutcome, AppError> {
    let net: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(amount), 0) FROM xp_transactions WHERE source_type = ? AND source_id = ?",
    )
    .bind(source_type)
    .bind(source_id)
    .fetch_one(&mut *conn)
    .await?;

    if net > 0 {
        apply_xp(conn, -net, source_type, source_id, reason).await
    } else {
        current_outcome(conn).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_thresholds() {
        assert_eq!(xp_for_level(1), 0);
        assert_eq!(xp_for_level(2), 100);
        assert_eq!(xp_for_level(3), 400);
        assert_eq!(xp_for_level(4), 900);
    }

    #[test]
    fn level_from_xp() {
        assert_eq!(level_for_xp(0), 1);
        assert_eq!(level_for_xp(99), 1);
        assert_eq!(level_for_xp(100), 2);
        assert_eq!(level_for_xp(399), 2);
        assert_eq!(level_for_xp(400), 3);
    }

    #[test]
    fn view_progress() {
        let v = build_view(2, 250);
        assert_eq!(v.xp_into_level, 150);
        assert_eq!(v.xp_for_next, 300); // level 2 spans 100..400
    }
}
