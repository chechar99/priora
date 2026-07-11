use chrono::Utc;
use sqlx::SqlitePool;

use crate::error::{AppError, AppResult};
use crate::models::{Namespace, NamespaceMember, User};

pub const MEMBER_STATUS_PENDING: &str = "pending";
pub const MEMBER_STATUS_ACTIVE: &str = "active";
pub const MEMBER_STATUS_DISABLED: &str = "disabled";
pub const MEMBER_STATUS_REJECTED: &str = "rejected";

pub const MEMBER_ROLE_REGULAR: &str = "regular";
pub const MEMBER_ROLE_PROPONENT: &str = "proponent";
pub const MEMBER_ROLE_SPACE_ADMIN: &str = "space_admin";

pub async fn get_membership(
    pool: &SqlitePool,
    namespace_id: &str,
    user_id: &str,
) -> AppResult<Option<NamespaceMember>> {
    Ok(
        sqlx::query_as::<_, NamespaceMember>(
            "SELECT * FROM namespace_members WHERE namespace_id = ? AND user_id = ?",
        )
        .bind(namespace_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?,
    )
}

pub fn is_global_admin(user: &User) -> bool {
    user.role == "admin"
}

pub fn is_active_space_admin(membership: Option<&NamespaceMember>) -> bool {
    membership.is_some_and(|m| {
        m.status == MEMBER_STATUS_ACTIVE && m.role == MEMBER_ROLE_SPACE_ADMIN
    })
}

pub fn can_manage_space(user: &User, membership: Option<&NamespaceMember>) -> bool {
    is_global_admin(user) || is_active_space_admin(membership)
}

pub fn can_create_in_space(user: &User, membership: Option<&NamespaceMember>) -> bool {
    if is_global_admin(user) || user.role == "proponent" {
        return true;
    }
    membership.is_some_and(|m| {
        m.status == MEMBER_STATUS_ACTIVE
            && (m.role == MEMBER_ROLE_PROPONENT || m.role == MEMBER_ROLE_SPACE_ADMIN)
    })
}

pub fn is_active_member(membership: Option<&NamespaceMember>) -> bool {
    membership.is_some_and(|m| m.status == MEMBER_STATUS_ACTIVE)
}

/// When approval is off, everyone participates. When on, only active members.
pub fn ranking_counts(ns: &Namespace, user: &User, membership: Option<&NamespaceMember>) -> bool {
    if is_global_admin(user) {
        return true;
    }
    if !ns.require_member_approval {
        return true;
    }
    is_active_member(membership)
}

pub fn can_comment(ns: &Namespace, user: &User, membership: Option<&NamespaceMember>) -> bool {
    if is_global_admin(user) {
        return true;
    }
    if !ns.require_member_approval {
        return true;
    }
    is_active_member(membership)
}

pub async fn request_membership(
    pool: &SqlitePool,
    namespace_id: &str,
    user_id: &str,
) -> AppResult<NamespaceMember> {
    let existing = get_membership(pool, namespace_id, user_id).await?;
    let now = Utc::now();

    if let Some(m) = existing {
        match m.status.as_str() {
            MEMBER_STATUS_ACTIVE => return Ok(m),
            MEMBER_STATUS_PENDING => return Ok(m),
            MEMBER_STATUS_DISABLED => {
                return Err(AppError::BadRequest(
                    "your access to this space is disabled".into(),
                ));
            }
            MEMBER_STATUS_REJECTED => {
                sqlx::query(
                    "UPDATE namespace_members
                     SET status = ?, requested_at = ?, reviewed_at = NULL, reviewed_by = NULL
                     WHERE namespace_id = ? AND user_id = ?",
                )
                .bind(MEMBER_STATUS_PENDING)
                .bind(now)
                .bind(namespace_id)
                .bind(user_id)
                .execute(pool)
                .await?;
            }
            _ => {
                return Err(AppError::BadRequest("invalid membership status".into()));
            }
        }
    } else {
        sqlx::query(
            "INSERT INTO namespace_members
             (namespace_id, user_id, role, status, requested_at)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(namespace_id)
        .bind(user_id)
        .bind(MEMBER_ROLE_REGULAR)
        .bind(MEMBER_STATUS_PENDING)
        .bind(now)
        .execute(pool)
        .await?;
    }

    get_membership(pool, namespace_id, user_id)
        .await?
        .ok_or_else(|| AppError::Internal("membership missing after request".into()))
}

pub fn validate_member_status(status: &str) -> AppResult<()> {
    match status {
        MEMBER_STATUS_PENDING
        | MEMBER_STATUS_ACTIVE
        | MEMBER_STATUS_DISABLED
        | MEMBER_STATUS_REJECTED => Ok(()),
        _ => Err(AppError::BadRequest(
            "status must be pending, active, disabled, or rejected".into(),
        )),
    }
}

pub fn validate_member_role(role: &str) -> AppResult<()> {
    match role {
        MEMBER_ROLE_REGULAR | MEMBER_ROLE_PROPONENT | MEMBER_ROLE_SPACE_ADMIN => Ok(()),
        _ => Err(AppError::BadRequest(
            "role must be regular, proponent, or space_admin".into(),
        )),
    }
}
