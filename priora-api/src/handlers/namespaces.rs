use std::sync::Arc;

use axum::extract::{Path, State};
use axum::Json;
use chrono::Utc;
use uuid::Uuid;

use crate::auth::is_admin;
use crate::db::fetch_namespace_by_slug;
use crate::error::{AppError, AppResult};
use crate::handlers::{AppState, AuthSession};
use crate::membership::{can_manage_space, get_membership};
use crate::models::{CreateNamespaceRequest, Namespace, UpdateNamespaceRequest};

pub async fn list(State(state): State<Arc<AppState>>) -> AppResult<Json<Vec<Namespace>>> {
    let rows = sqlx::query_as::<_, Namespace>(
        "SELECT * FROM namespaces ORDER BY name ASC",
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rows))
}

pub async fn get_one(
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
) -> AppResult<Json<Namespace>> {
    let ns = fetch_namespace_by_slug(&state.pool, &slug).await?;
    Ok(Json(ns))
}

pub async fn create(
    State(state): State<Arc<AppState>>,
    session: AuthSession,
    Json(body): Json<CreateNamespaceRequest>,
) -> AppResult<Json<Namespace>> {
    if !is_admin(&session.user) {
        return Err(AppError::Forbidden);
    }

    let name = body.name.trim();
    let slug = body.slug.trim().to_lowercase();

    if name.is_empty() {
        return Err(AppError::BadRequest("name is required".into()));
    }
    if name.len() > 100 {
        return Err(AppError::BadRequest("name is too long".into()));
    }
    if !is_valid_slug(&slug) {
        return Err(AppError::BadRequest(
            "slug must be 2–64 chars: lowercase letters, numbers, and hyphens".into(),
        ));
    }

    let existing = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM namespaces WHERE slug = ?")
        .bind(&slug)
        .fetch_one(&state.pool)
        .await?;
    if existing > 0 {
        return Err(AppError::BadRequest("slug already exists".into()));
    }

    let id = Uuid::new_v4().to_string();
    let now = Utc::now();
    sqlx::query(
        "INSERT INTO namespaces (id, slug, name, require_member_approval, created_at)
         VALUES (?, ?, ?, 0, ?)",
    )
    .bind(&id)
    .bind(&slug)
    .bind(name)
    .bind(now)
    .execute(&state.pool)
    .await?;

    let ns = fetch_namespace_by_slug(&state.pool, &slug).await?;
    Ok(Json(ns))
}

pub async fn update(
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
    session: AuthSession,
    Json(body): Json<UpdateNamespaceRequest>,
) -> AppResult<Json<Namespace>> {
    let ns = fetch_namespace_by_slug(&state.pool, &slug).await?;
    let my = get_membership(&state.pool, &ns.id, &session.user.id).await?;
    if !can_manage_space(&session.user, my.as_ref()) {
        return Err(AppError::Forbidden);
    }

    if let Some(require) = body.require_member_approval {
        sqlx::query("UPDATE namespaces SET require_member_approval = ? WHERE id = ?")
            .bind(require)
            .bind(&ns.id)
            .execute(&state.pool)
            .await?;
    }

    let updated = fetch_namespace_by_slug(&state.pool, &slug).await?;
    Ok(Json(updated))
}

fn is_valid_slug(slug: &str) -> bool {
    let len = slug.len();
    if !(2..=64).contains(&len) {
        return false;
    }
    let mut chars = slug.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_lowercase() && !first.is_ascii_digit() {
        return false;
    }
    chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !slug.contains("--")
        && !slug.ends_with('-')
}
