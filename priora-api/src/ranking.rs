use std::collections::HashMap;

use sqlx::SqlitePool;

use crate::error::AppResult;

/// Minimum distinct rankers before consensus/conflict labels apply.
pub const MIN_RANKERS_FOR_AGREEMENT: usize = 3;

/// Relative-position stddev below this → consensus.
const CONSENSUS_STDDEV: f64 = 0.22;
/// Relative-position stddev above this → polarized.
const POLARIZED_STDDEV: f64 = 0.38;

#[derive(Debug, Clone)]
pub struct ProposalRankingStats {
    pub score: i64,
    pub rankers_count: i64,
    pub avg_position: f64,
    pub relative_stddev: f64,
    /// "consensus" | "polarized" | None (mixed / insufficient data)
    pub agreement: Option<&'static str>,
    pub top3_count: i64,
}

async fn fetch_eligible_ranking_rows(
    pool: &SqlitePool,
    namespace_id: &str,
) -> AppResult<Vec<(String, String, i64)>> {
    let require_approval: bool =
        sqlx::query_scalar("SELECT require_member_approval FROM namespaces WHERE id = ?")
            .bind(namespace_id)
            .fetch_optional(pool)
            .await?
            .unwrap_or(false);

    let rows = if require_approval {
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

    Ok(rows)
}

fn group_by_user(rows: Vec<(String, String, i64)>) -> HashMap<String, Vec<(String, i64)>> {
    let mut grouped: HashMap<String, Vec<(String, i64)>> = HashMap::new();
    for (user_id, proposal_id, position) in rows {
        grouped
            .entry(user_id)
            .or_default()
            .push((proposal_id, position));
    }
    grouped
}

pub async fn compute_borda_scores(
    pool: &SqlitePool,
    namespace_id: &str,
) -> AppResult<HashMap<String, i64>> {
    let rows = fetch_eligible_ranking_rows(pool, namespace_id).await?;
    let grouped = group_by_user(rows);

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

/// Per-proposal Borda score plus agreement (consensus vs polarized).
pub async fn compute_ranking_stats(
    pool: &SqlitePool,
    namespace_id: &str,
) -> AppResult<HashMap<String, ProposalRankingStats>> {
    let rows = fetch_eligible_ranking_rows(pool, namespace_id).await?;
    let grouped = group_by_user(rows);

    // proposal_id -> (score, relative positions, absolute positions, top3 flags)
    let mut scores: HashMap<String, i64> = HashMap::new();
    let mut relatives: HashMap<String, Vec<f64>> = HashMap::new();
    let mut positions: HashMap<String, Vec<f64>> = HashMap::new();
    let mut top3: HashMap<String, i64> = HashMap::new();

    for items in grouped.values() {
        let n = items.len() as i64;
        let denom = (n - 1).max(1) as f64;
        for (proposal_id, position) in items {
            let points = n - position;
            *scores.entry(proposal_id.clone()).or_insert(0) += points;
            let rel = (*position as f64) / denom;
            relatives
                .entry(proposal_id.clone())
                .or_default()
                .push(rel);
            positions
                .entry(proposal_id.clone())
                .or_default()
                .push(*position as f64);
            if *position < 3 {
                *top3.entry(proposal_id.clone()).or_insert(0) += 1;
            }
        }
    }

    let mut out = HashMap::new();
    for (proposal_id, score) in scores {
        let rels = relatives.remove(&proposal_id).unwrap_or_default();
        let pos = positions.remove(&proposal_id).unwrap_or_default();
        let rankers = rels.len();
        let avg_position = if pos.is_empty() {
            0.0
        } else {
            pos.iter().sum::<f64>() / pos.len() as f64
        };
        let relative_stddev = stddev(&rels);
        let agreement = classify_agreement(rankers, relative_stddev);
        out.insert(
            proposal_id.clone(),
            ProposalRankingStats {
                score,
                rankers_count: rankers as i64,
                avg_position,
                relative_stddev,
                agreement,
                top3_count: top3.remove(&proposal_id).unwrap_or(0),
            },
        );
    }

    Ok(out)
}

fn stddev(values: &[f64]) -> f64 {
    let n = values.len();
    if n < 2 {
        return 0.0;
    }
    let mean = values.iter().sum::<f64>() / n as f64;
    let var = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n as f64;
    var.sqrt()
}

fn classify_agreement(rankers: usize, relative_stddev: f64) -> Option<&'static str> {
    if rankers < MIN_RANKERS_FOR_AGREEMENT {
        return None;
    }
    if relative_stddev <= CONSENSUS_STDDEV {
        Some("consensus")
    } else if relative_stddev >= POLARIZED_STDDEV {
        Some("polarized")
    } else {
        None
    }
}

/// Distinct users whose rankings currently count toward Borda.
pub async fn count_eligible_rankers(pool: &SqlitePool, namespace_id: &str) -> AppResult<i64> {
    let rows = fetch_eligible_ranking_rows(pool, namespace_id).await?;
    let mut users = std::collections::HashSet::new();
    for (user_id, _, _) in rows {
        users.insert(user_id);
    }
    Ok(users.len() as i64)
}
