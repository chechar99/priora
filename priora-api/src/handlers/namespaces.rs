use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::Json;
use chrono::Utc;
use serde::Deserialize;
use uuid::Uuid;

use crate::auth::is_admin;
use crate::db::fetch_namespace_by_slug;
use crate::error::{AppError, AppResult};
use crate::handlers::proposals::NamespacePath;
use crate::handlers::{AppState, AuthSession, OptionalAuthSession};
use crate::membership::{can_manage_space, get_membership};
use crate::models::{CreateNamespaceRequest, Namespace, NamespaceInvite, UpdateNamespaceRequest};

#[derive(Debug, Deserialize)]
pub struct ListNamespacesQuery {
    pub include_hidden: Option<bool>,
}

fn generate_invite_code() -> String {
    let raw = Uuid::new_v4().simple().to_string();
    raw[..12].to_string()
}

fn normalize_description(description: Option<String>) -> Result<String, AppError> {
    let text = description.unwrap_or_default().trim().to_string();
    if text.len() > 500 {
        return Err(AppError::BadRequest("description is too long".into()));
    }
    Ok(text)
}

pub async fn list(
    State(state): State<Arc<AppState>>,
    auth: OptionalAuthSession,
    Query(query): Query<ListNamespacesQuery>,
) -> AppResult<Json<Vec<Namespace>>> {
    let include_hidden = query.include_hidden == Some(true)
        && auth
            .session
            .as_ref()
            .is_some_and(|s| is_admin(&s.user));

    let rows = if include_hidden {
        sqlx::query_as::<_, Namespace>("SELECT * FROM namespaces ORDER BY name ASC")
            .fetch_all(&state.pool)
            .await?
    } else {
        sqlx::query_as::<_, Namespace>(
            "SELECT * FROM namespaces WHERE is_hidden = 0 ORDER BY name ASC",
        )
        .fetch_all(&state.pool)
        .await?
    };

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
    let description = normalize_description(body.description)?;
    let is_hidden = body.is_hidden.unwrap_or(false);

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
    let invite_code = generate_invite_code();
    let now = Utc::now();
    sqlx::query(
        "INSERT INTO namespaces (id, slug, name, description, is_hidden, require_member_approval, invite_code, created_at)
         VALUES (?, ?, ?, ?, ?, 0, ?, ?)",
    )
    .bind(&id)
    .bind(&slug)
    .bind(name)
    .bind(&description)
    .bind(is_hidden)
    .bind(&invite_code)
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

    if let Some(name) = body.name {
        let name = name.trim();
        if name.is_empty() {
            return Err(AppError::BadRequest("name is required".into()));
        }
        if name.len() > 100 {
            return Err(AppError::BadRequest("name is too long".into()));
        }
        sqlx::query("UPDATE namespaces SET name = ? WHERE id = ?")
            .bind(name)
            .bind(&ns.id)
            .execute(&state.pool)
            .await?;
    }

    if let Some(description) = body.description {
        let description = normalize_description(Some(description))?;
        sqlx::query("UPDATE namespaces SET description = ? WHERE id = ?")
            .bind(&description)
            .bind(&ns.id)
            .execute(&state.pool)
            .await?;
    }

    if let Some(is_hidden) = body.is_hidden {
        if !is_admin(&session.user) {
            return Err(AppError::Forbidden);
        }
        sqlx::query("UPDATE namespaces SET is_hidden = ? WHERE id = ?")
            .bind(is_hidden)
            .bind(&ns.id)
            .execute(&state.pool)
            .await?;
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

pub async fn get_invite(
    State(state): State<Arc<AppState>>,
    Path(ns_path): Path<NamespacePath>,
    session: AuthSession,
) -> AppResult<Json<NamespaceInvite>> {
    let ns = fetch_namespace_by_slug(&state.pool, &ns_path.namespace).await?;
    let my = get_membership(&state.pool, &ns.id, &session.user.id).await?;
    if !can_manage_space(&session.user, my.as_ref()) {
        return Err(AppError::Forbidden);
    }

    Ok(Json(NamespaceInvite {
        invite_code: ns.invite_code.clone(),
        invite_path: format!("/for/{}?invite={}", ns.slug, ns.invite_code),
    }))
}

pub async fn regenerate_invite(
    State(state): State<Arc<AppState>>,
    Path(ns_path): Path<NamespacePath>,
    session: AuthSession,
) -> AppResult<Json<NamespaceInvite>> {
    let ns = fetch_namespace_by_slug(&state.pool, &ns_path.namespace).await?;
    let my = get_membership(&state.pool, &ns.id, &session.user.id).await?;
    if !can_manage_space(&session.user, my.as_ref()) {
        return Err(AppError::Forbidden);
    }

    let invite_code = generate_invite_code();
    sqlx::query("UPDATE namespaces SET invite_code = ? WHERE id = ?")
        .bind(&invite_code)
        .bind(&ns.id)
        .execute(&state.pool)
        .await?;

    Ok(Json(NamespaceInvite {
        invite_code: invite_code.clone(),
        invite_path: format!("/for/{}?invite={}", ns.slug, invite_code),
    }))
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
