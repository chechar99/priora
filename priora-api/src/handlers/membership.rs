use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::Json;
use chrono::Utc;
use serde::Deserialize;

use crate::auth::ensure_profile;
use crate::db::fetch_namespace_by_slug;
use crate::error::{AppError, AppResult};
use crate::handlers::proposals::NamespacePath;
use crate::handlers::{AppState, AuthSession, OptionalAuthSession};
use crate::membership::{
    can_comment, can_create_in_space, can_manage_space, get_membership, is_global_admin,
    ranking_counts, request_membership, validate_member_role, validate_member_status,
    MEMBER_ROLE_SPACE_ADMIN, MEMBER_STATUS_ACTIVE, MEMBER_STATUS_DISABLED, MEMBER_STATUS_PENDING,
    MEMBER_STATUS_REJECTED,
};
use crate::models::{
    MembershipMeResponse, NamespaceMember, NamespaceMemberPublic, UpdateMemberRequest, User,
};

#[derive(Deserialize)]
pub struct MembersQuery {
    #[serde(default)]
    status: Option<String>,
}

#[derive(Deserialize)]
pub struct MemberPath {
    pub namespace: String,
    pub user_id: String,
}

pub async fn me(
    State(state): State<Arc<AppState>>,
    Path(ns_path): Path<NamespacePath>,
    auth: OptionalAuthSession,
) -> AppResult<Json<MembershipMeResponse>> {
    let ns = fetch_namespace_by_slug(&state.pool, &ns_path.namespace).await?;

    let Some(session) = auth.session else {
        return Ok(Json(MembershipMeResponse {
            require_member_approval: ns.require_member_approval,
            membership: None,
            can_comment: false,
            ranking_counts: false,
            can_manage_space: false,
            can_create_proposal: false,
        }));
    };

    let membership = get_membership(&state.pool, &ns.id, &session.user.id).await?;
    let m_ref = membership.as_ref();

    Ok(Json(MembershipMeResponse {
        require_member_approval: ns.require_member_approval,
        membership: membership
            .clone()
            .map(|m| member_to_public(&session.user, m)),
        can_comment: session.user.profile_complete
            && can_comment(&ns, &session.user, m_ref),
        ranking_counts: ranking_counts(&ns, &session.user, m_ref),
        can_manage_space: can_manage_space(&session.user, m_ref),
        can_create_proposal: can_create_in_space(&session.user, m_ref),
    }))
}

pub async fn request(
    State(state): State<Arc<AppState>>,
    Path(ns_path): Path<NamespacePath>,
    session: AuthSession,
) -> AppResult<Json<MembershipMeResponse>> {
    ensure_profile(&session.user)?;
    let ns = fetch_namespace_by_slug(&state.pool, &ns_path.namespace).await?;

    if !ns.require_member_approval {
        return Err(AppError::BadRequest(
            "this space does not require member approval".into(),
        ));
    }

    request_membership(&state.pool, &ns.id, &session.user.id).await?;

    me(
        State(state),
        Path(ns_path),
        OptionalAuthSession {
            session: Some(session),
        },
    )
    .await
}

pub async fn list(
    State(state): State<Arc<AppState>>,
    Path(ns_path): Path<NamespacePath>,
    Query(query): Query<MembersQuery>,
    session: AuthSession,
) -> AppResult<Json<Vec<NamespaceMemberPublic>>> {
    let ns = fetch_namespace_by_slug(&state.pool, &ns_path.namespace).await?;
    let my = get_membership(&state.pool, &ns.id, &session.user.id).await?;
    if !can_manage_space(&session.user, my.as_ref()) {
        return Err(AppError::Forbidden);
    }

    if let Some(ref status) = query.status {
        validate_member_status(status)?;
    }

    let members = if let Some(status) = query.status.as_deref() {
        sqlx::query_as::<_, NamespaceMember>(
            "SELECT * FROM namespace_members
             WHERE namespace_id = ? AND status = ?
             ORDER BY requested_at ASC",
        )
        .bind(&ns.id)
        .bind(status)
        .fetch_all(&state.pool)
        .await?
    } else {
        sqlx::query_as::<_, NamespaceMember>(
            "SELECT * FROM namespace_members
             WHERE namespace_id = ?
             ORDER BY
               CASE status
                 WHEN 'pending' THEN 0
                 WHEN 'active' THEN 1
                 WHEN 'rejected' THEN 2
                 WHEN 'disabled' THEN 3
                 ELSE 4
               END,
               requested_at ASC",
        )
        .bind(&ns.id)
        .fetch_all(&state.pool)
        .await?
    };

    let mut result = Vec::with_capacity(members.len());
    for m in members {
        let user = crate::auth::get_user_by_id(&state.pool, &m.user_id).await?;
        result.push(member_to_public(&user, m));
    }
    Ok(Json(result))
}

pub async fn update(
    State(state): State<Arc<AppState>>,
    Path(path): Path<MemberPath>,
    session: AuthSession,
    Json(body): Json<UpdateMemberRequest>,
) -> AppResult<Json<NamespaceMemberPublic>> {
    let ns = fetch_namespace_by_slug(&state.pool, &path.namespace).await?;
    let my = get_membership(&state.pool, &ns.id, &session.user.id).await?;
    if !can_manage_space(&session.user, my.as_ref()) {
        return Err(AppError::Forbidden);
    }

    let existing = get_membership(&state.pool, &ns.id, &path.user_id).await?;

    if path.user_id == session.user.id {
        return Err(AppError::BadRequest(
            "cannot change your own membership".into(),
        ));
    }

    // Ensure target user exists.
    let target_user = crate::auth::get_user_by_id(&state.pool, &path.user_id).await?;

    let existing = match existing {
        Some(m) => m,
        None => {
            if !is_global_admin(&session.user) {
                return Err(AppError::NotFound);
            }
            let now = Utc::now();
            sqlx::query(
                "INSERT INTO namespace_members
                 (namespace_id, user_id, role, status, requested_at, reviewed_at, reviewed_by)
                 VALUES (?, ?, 'regular', 'active', ?, ?, ?)",
            )
            .bind(&ns.id)
            .bind(&path.user_id)
            .bind(now)
            .bind(now)
            .bind(&session.user.id)
            .execute(&state.pool)
            .await?;
            get_membership(&state.pool, &ns.id, &path.user_id)
                .await?
                .ok_or_else(|| AppError::Internal("membership missing after insert".into()))?
        }
    };

    let new_status = body.status.as_deref().unwrap_or(&existing.status);
    let new_role = body.role.as_deref().unwrap_or(&existing.role);
    validate_member_status(new_status)?;
    validate_member_role(new_role)?;

    // Only global admins can assign/revoke space_admin.
    if new_role != existing.role.as_str()
        && (new_role == MEMBER_ROLE_SPACE_ADMIN || existing.role == MEMBER_ROLE_SPACE_ADMIN)
        && !is_global_admin(&session.user)
    {
        return Err(AppError::Forbidden);
    }

    // Space admins may only manage regular/proponent members (approve/reject/disable).
    if !is_global_admin(&session.user) && existing.role == MEMBER_ROLE_SPACE_ADMIN {
        return Err(AppError::Forbidden);
    }

    let now = Utc::now();
    let status_changed = new_status != existing.status;
    let (reviewed_at, reviewed_by) = if status_changed
        && matches!(
            new_status,
            MEMBER_STATUS_ACTIVE | MEMBER_STATUS_REJECTED | MEMBER_STATUS_DISABLED
        ) {
        (Some(now), Some(session.user.id.clone()))
    } else if status_changed && new_status == MEMBER_STATUS_PENDING {
        (None, None)
    } else {
        (existing.reviewed_at, existing.reviewed_by.clone())
    };

    sqlx::query(
        "UPDATE namespace_members
         SET role = ?, status = ?, reviewed_at = ?, reviewed_by = ?
         WHERE namespace_id = ? AND user_id = ?",
    )
    .bind(new_role)
    .bind(new_status)
    .bind(reviewed_at)
    .bind(&reviewed_by)
    .bind(&ns.id)
    .bind(&path.user_id)
    .execute(&state.pool)
    .await?;

    let updated = get_membership(&state.pool, &ns.id, &path.user_id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(member_to_public(&target_user, updated)))
}

fn member_to_public(user: &User, m: NamespaceMember) -> NamespaceMemberPublic {
    NamespaceMemberPublic {
        user_id: m.user_id,
        name: user.name.clone(),
        email: user.email.clone(),
        picture_url: user.picture_url.clone(),
        street: user.street.clone(),
        city: user.city.clone(),
        role: m.role,
        status: m.status,
        requested_at: m.requested_at,
        reviewed_at: m.reviewed_at,
    }
}
