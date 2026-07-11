use std::sync::Arc;

use axum::extract::{Path, State};
use axum::Json;
use chrono::Utc;

use crate::auth::{get_user_by_id, is_admin};
use crate::error::{AppError, AppResult};
use crate::handlers::{AppState, AuthSession};
use crate::models::{UpdateProfileRequest, UpdateRoleRequest, User};

const VALID_ROLES: &[&str] = &["regular", "proponent", "admin"];

pub async fn get_me(session: AuthSession) -> Json<User> {
    Json(session.user)
}

pub async fn update_me(
    State(state): State<Arc<AppState>>,
    session: AuthSession,
    Json(body): Json<UpdateProfileRequest>,
) -> AppResult<Json<User>> {
    if body.street.trim().len() < 5 {
        return Err(AppError::BadRequest(
            "street must be at least 5 characters".into(),
        ));
    }
    if body.city.trim().is_empty() {
        return Err(AppError::BadRequest("city is required".into()));
    }

    let now = Utc::now();
    sqlx::query(
        "UPDATE users SET street = ?, floor_apt = ?, city = ?, postal_code = ?,
         profile_complete = 1, updated_at = ? WHERE id = ?",
    )
    .bind(body.street.trim())
    .bind(body.floor_apt)
    .bind(body.city.trim())
    .bind(body.postal_code)
    .bind(now)
    .bind(&session.user.id)
    .execute(&state.pool)
    .await?;

    let updated = get_user_by_id(&state.pool, &session.user.id).await?;
    Ok(Json(updated))
}

pub async fn list(
    State(state): State<Arc<AppState>>,
    session: AuthSession,
) -> AppResult<Json<Vec<User>>> {
    if !is_admin(&session.user) {
        return Err(AppError::Forbidden);
    }

    let rows = sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY name ASC")
        .fetch_all(&state.pool)
        .await?;
    Ok(Json(rows))
}

pub async fn update_role(
    State(state): State<Arc<AppState>>,
    session: AuthSession,
    Path(id): Path<String>,
    Json(body): Json<UpdateRoleRequest>,
) -> AppResult<Json<User>> {
    if !is_admin(&session.user) {
        return Err(AppError::Forbidden);
    }

    let role = body.role.trim();
    if !VALID_ROLES.contains(&role) {
        return Err(AppError::BadRequest(
            "role must be regular, proponent, or admin".into(),
        ));
    }

    if id == session.user.id {
        return Err(AppError::BadRequest(
            "cannot change your own role".into(),
        ));
    }

    let target = get_user_by_id(&state.pool, &id).await?;

    if target.role == "admin" && role != "admin" {
        let admin_count =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE role = 'admin'")
                .fetch_one(&state.pool)
                .await?;
        if admin_count <= 1 {
            return Err(AppError::BadRequest(
                "cannot demote the last administrator".into(),
            ));
        }
    }

    let now = Utc::now();
    sqlx::query("UPDATE users SET role = ?, updated_at = ? WHERE id = ?")
        .bind(role)
        .bind(now)
        .bind(&id)
        .execute(&state.pool)
        .await?;

    let updated = get_user_by_id(&state.pool, &id).await?;
    Ok(Json(updated))
}
