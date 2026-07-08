use std::collections::HashMap;

use sqlx::SqlitePool;

use crate::error::AppResult;

pub async fn compute_borda_scores(
    pool: &SqlitePool,
    namespace_id: &str,
) -> AppResult<HashMap<String, i64>> {
    let rows = sqlx::query_as::<_, (String, String, i64)>(
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
    .await?;

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
