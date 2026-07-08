use std::sync::Arc;

use axum::extract::State;
use axum::Json;
use chrono::Utc;

use crate::auth::ensure_profile;
use crate::db::fetch_namespace_by_slug;
use crate::error::{AppError, AppResult};
use crate::handlers::{proposals::NamespacePath, AppState, AuthSession};
use crate::models::{RankingResponse, SaveRankingRequest};

pub async fn get_my(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(ns_path): axum::extract::Path<NamespacePath>,
    session: AuthSession,
) -> AppResult<Json<RankingResponse>> {
    ensure_profile(&session.user)?;
    let ns = fetch_namespace_by_slug(&state.pool, &ns_path.namespace).await?;

    let rows = sqlx::query_as::<_, (String,)>(
        "SELECT proposal_id FROM user_rankings
         WHERE user_id = ? AND namespace_id = ?
         ORDER BY position ASC",
    )
    .bind(&session.user.id)
    .bind(&ns.id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(RankingResponse {
        proposal_ids: rows.into_iter().map(|r| r.0).collect(),
    }))
}

pub async fn save_my(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(ns_path): axum::extract::Path<NamespacePath>,
    session: AuthSession,
    Json(body): Json<SaveRankingRequest>,
) -> AppResult<Json<RankingResponse>> {
    ensure_profile(&session.user)?;
    let ns = fetch_namespace_by_slug(&state.pool, &ns_path.namespace).await?;

    let active: Vec<(String,)> = sqlx::query_as(
        "SELECT id FROM proposals
         WHERE namespace_id = ? AND status IN ('activa', 'en_analisis')",
    )
    .bind(&ns.id)
    .fetch_all(&state.pool)
    .await?;

    let active_ids: std::collections::HashSet<String> =
        active.into_iter().map(|r| r.0).collect();

    for pid in &body.proposal_ids {
        if !active_ids.contains(pid) {
            return Err(AppError::BadRequest(format!(
                "invalid or inactive proposal: {pid}"
            )));
        }
    }

    let mut tx = state.pool.begin().await?;
    sqlx::query("DELETE FROM user_rankings WHERE user_id = ? AND namespace_id = ?")
        .bind(&session.user.id)
        .bind(&ns.id)
        .execute(&mut *tx)
        .await?;

    let now = Utc::now();
    for (position, proposal_id) in body.proposal_ids.iter().enumerate() {
        sqlx::query(
            "INSERT INTO user_rankings (user_id, namespace_id, proposal_id, position, updated_at)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&session.user.id)
        .bind(&ns.id)
        .bind(proposal_id)
        .bind(position as i64)
        .bind(now)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;

    get_my(
        State(state),
        axum::extract::Path(ns_path),
        session,
    )
    .await
}
