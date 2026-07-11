use std::collections::HashMap;

use sqlx::SqlitePool;

use crate::error::AppResult;

pub async fn compute_borda_scores(
    pool: &SqlitePool,
    namespace_id: &str,
) -> AppResult<HashMap<String, i64>> {
    let require_approval: bool =
        sqlx::query_scalar("SELECT require_member_approval FROM namespaces WHERE id = ?")
            .bind(namespace_id)
            .fetch_optional(pool)
            .await?
            .unwrap_or(false);

    let rows = if require_approval {
        // Only rankings from active members (and platform admins) count.
        sqlx::query_as::<_, (String, String, i64)>(
            "SELECT ur.user_id, ur.proposal_id, ur.position FROM user_rankings ur
             WHERE ur.namespace_id = ?
             AND ur.proposal_id IN (
               SELECT id FROM proposals
               WHERE namespace_id = ? AND status IN ('activa', 'en_analisis')
             )
             AND (
               ur.user_id IN (
                 SELECT user_id FROM namespace_members
                 WHERE namespace_id = ? AND status = 'active'
               )
               OR ur.user_id IN (SELECT id FROM users WHERE role = 'admin')
             )
             ORDER BY ur.user_id, ur.position",
        )
        .bind(namespace_id)
        .bind(namespace_id)
        .bind(namespace_id)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, (String, String, i64)>(
            "SELECT user_id, proposal_id, position FROM user_rankings
             WHERE namespace_id = ?
             AND proposal_id IN (
               SELECT id FROM proposals
               WHERE namespace_id = ? AND status IN ('activa', 'en_analisis')
             )
             ORDER BY user_id, position",
        )
        .bind(namespace_id)
        .bind(namespace_id)
        .fetch_all(pool)
        .await?
    };

    let mut grouped: HashMap<String, Vec<(String, i64)>> = HashMap::new();
    for (user_id, proposal_id, position) in rows {
        grouped
            .entry(user_id)
            .or_default()
            .push((proposal_id, position));
    }

    let mut scores: HashMap<String, i64> = HashMap::new();
    for items in grouped.values() {
        let n = items.len() as i64;
        for (proposal_id, position) in items {
            let points = n - position;
            *scores.entry(proposal_id.clone()).or_insert(0) += points;
        }
    }

    Ok(scores)
}
