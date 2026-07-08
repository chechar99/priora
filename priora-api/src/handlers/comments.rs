use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::Json;
use chrono::Utc;
use serde::Deserialize;
use uuid::Uuid;

use crate::auth::{ensure_profile, is_admin};
use crate::db::{fetch_namespace_by_slug, fetch_user_public};
use crate::error::{AppError, AppResult};
use crate::handlers::proposals::{fetch_proposal_in_namespace, ProposalPath};
use crate::handlers::{AppState, AuthSession};
use crate::models::{
    Comment, CommentWithAuthor, CommentsPage, CreateCommentRequest,
};

#[derive(Deserialize)]
pub struct CommentsQuery {
    #[serde(default = "default_limit")]
    limit: i64,
    #[serde(default)]
    offset: i64,
}

fn default_limit() -> i64 {
    10
}

pub async fn list(
    State(state): State<Arc<AppState>>,
    Path(path): Path<ProposalPath>,
    Query(query): Query<CommentsQuery>,
) -> AppResult<Json<CommentsPage>> {
    let ns = fetch_namespace_by_slug(&state.pool, &path.namespace).await?;
    fetch_proposal_in_namespace(&state.pool, &ns.id, &path.id).await?;

    let total: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM comments WHERE proposal_id = ?",
    )
    .bind(&path.id)
    .fetch_one(&state.pool)
    .await?;

    let comments = sqlx::query_as::<_, Comment>(
        "SELECT * FROM comments WHERE proposal_id = ? ORDER BY created_at DESC LIMIT ? OFFSET ?",
    )
    .bind(&path.id)
    .bind(query.limit)
    .bind(query.offset)
    .fetch_all(&state.pool)
    .await?;

    let mut result = Vec::new();
    for c in comments {
        let author = fetch_user_public(&state.pool, &c.author_id).await?;
        result.push(CommentWithAuthor { comment: c, author });
    }

    Ok(Json(CommentsPage {
        comments: result,
        total: total.0,
    }))
}

pub async fn create(
    State(state): State<Arc<AppState>>,
    Path(path): Path<ProposalPath>,
    session: AuthSession,
    Json(body): Json<CreateCommentRequest>,
) -> AppResult<Json<CommentWithAuthor>> {
    ensure_profile(&session.user)?;
    let ns = fetch_namespace_by_slug(&state.pool, &path.namespace).await?;
    let p = fetch_proposal_in_namespace(&state.pool, &ns.id, &path.id).await?;

    if p.status == "rechazada" {
        return Err(AppError::BadRequest(
            "cannot comment on rejected proposals".into(),
        ));
    }
    if body.content.trim().is_empty() {
        return Err(AppError::BadRequest("content required".into()));
    }

    let id = Uuid::new_v4().to_string();
    let now = Utc::now();
    sqlx::query(
        "INSERT INTO comments (id, proposal_id, author_id, content, created_at) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&path.id)
    .bind(&session.user.id)
    .bind(body.content.trim())
    .bind(now)
    .execute(&state.pool)
    .await?;

    let comment = sqlx::query_as::<_, Comment>("SELECT * FROM comments WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.pool)
        .await?;
    let author = fetch_user_public(&state.pool, &session.user.id).await?;

    Ok(Json(CommentWithAuthor { comment, author }))
}

#[derive(Deserialize)]
pub struct CommentDeletePath {
    pub namespace: String,
    pub id: String,
}

pub async fn delete_comment(
    State(state): State<Arc<AppState>>,
    Path(path): Path<CommentDeletePath>,
    session: AuthSession,
) -> AppResult<Json<serde_json::Value>> {
    let ns = fetch_namespace_by_slug(&state.pool, &path.namespace).await?;
    let user = &session.user;
    let comment = sqlx::query_as::<_, Comment>("SELECT * FROM comments WHERE id = ?")
        .bind(&path.id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or(AppError::NotFound)?;

    fetch_proposal_in_namespace(&state.pool, &ns.id, &comment.proposal_id).await?;

    if comment.author_id != user.id && !is_admin(user) {
        return Err(AppError::Forbidden);
    }

    sqlx::query("DELETE FROM comments WHERE id = ?")
        .bind(&path.id)
        .execute(&state.pool)
        .await?;

    Ok(Json(serde_json::json!({ "ok": true })))
}
