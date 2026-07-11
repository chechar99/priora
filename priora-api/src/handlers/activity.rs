use std::sync::Arc;

use axum::extract::{Path, State};
use axum::Json;
use chrono::{DateTime, Utc};

use crate::db::fetch_namespace_by_slug;
use crate::error::AppResult;
use crate::handlers::proposals::NamespacePath;
use crate::handlers::{AppState, AuthSession};
use crate::models::{
    ActivityComment, ActivityProposal, ActivityRankingItem, MyActivityResponse,
};
use crate::ranking::compute_borda_scores;

pub async fn my_activity(
    State(state): State<Arc<AppState>>,
    Path(ns_path): Path<NamespacePath>,
    session: AuthSession,
) -> AppResult<Json<MyActivityResponse>> {
    let ns = fetch_namespace_by_slug(&state.pool, &ns_path.namespace).await?;
    let user_id = &session.user.id;

    let proposal_rows: Vec<(String, String, String, DateTime<Utc>)> = sqlx::query_as(
        "SELECT id, title, status, created_at FROM proposals
         WHERE namespace_id = ? AND author_id = ?
         ORDER BY created_at DESC
         LIMIT 20",
    )
    .bind(&ns.id)
    .bind(user_id)
    .fetch_all(&state.pool)
    .await?;

    let scores = compute_borda_scores(&state.pool, &ns.id).await?;

    let ranked: Vec<(String,)> = sqlx::query_as(
        "SELECT id FROM proposals
         WHERE namespace_id = ? AND status IN ('activa', 'en_analisis')
         ORDER BY created_at ASC",
    )
    .bind(&ns.id)
    .fetch_all(&state.pool)
    .await?;

    let mut ranked_ids: Vec<(String, i64)> = ranked
        .into_iter()
        .map(|(id,)| {
            let score = scores.get(&id).copied().unwrap_or(0);
            (id, score)
        })
        .collect();
    ranked_ids.sort_by(|a, b| b.1.cmp(&a.1));

    let rank_of = |id: &str| -> Option<i64> {
        ranked_ids
            .iter()
            .position(|(pid, _)| pid == id)
            .map(|i| (i + 1) as i64)
    };

    let proposals: Vec<ActivityProposal> = proposal_rows
        .into_iter()
        .map(|(id, title, status, created_at)| {
            let score = scores.get(&id).copied().unwrap_or(0);
            let rank_position = if status == "activa" || status == "en_analisis" {
                rank_of(&id)
            } else {
                None
            };
            ActivityProposal {
                id,
                title,
                status,
                rank_position,
                score,
                created_at,
            }
        })
        .collect();

    let ranking_rows: Vec<(String, String, i64)> = sqlx::query_as(
        "SELECT ur.proposal_id, p.title, ur.position
         FROM user_rankings ur
         INNER JOIN proposals p ON p.id = ur.proposal_id
         WHERE ur.user_id = ? AND ur.namespace_id = ?
         ORDER BY ur.position ASC",
    )
    .bind(user_id)
    .bind(&ns.id)
    .fetch_all(&state.pool)
    .await?;

    let n = ranking_rows.len() as i64;
    let ranking: Vec<ActivityRankingItem> = ranking_rows
        .into_iter()
        .map(|(proposal_id, title, position)| ActivityRankingItem {
            proposal_id,
            title,
            position,
            points: n - position,
        })
        .collect();

    let comment_rows: Vec<(String, String, String, String, DateTime<Utc>)> = sqlx::query_as(
        "SELECT c.id, c.content, c.proposal_id, p.title, c.created_at
         FROM comments c
         INNER JOIN proposals p ON p.id = c.proposal_id
         WHERE c.author_id = ? AND p.namespace_id = ?
         ORDER BY c.created_at DESC
         LIMIT 20",
    )
    .bind(user_id)
    .bind(&ns.id)
    .fetch_all(&state.pool)
    .await?;

    let comments: Vec<ActivityComment> = comment_rows
        .into_iter()
        .map(
            |(id, content, proposal_id, proposal_title, created_at)| ActivityComment {
                id,
                content,
                proposal_id,
                proposal_title,
                created_at,
            },
        )
        .collect();

    Ok(Json(MyActivityResponse {
        proposals,
        ranking,
        comments,
    }))
}
