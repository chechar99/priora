use std::sync::Arc;

use axum::extract::State;
use axum::Json;
use chrono::Utc;

use crate::auth::{is_admin, VALID_DEFAULT_ROLES};
use crate::error::{AppError, AppResult};
use crate::handlers::{AppState, AuthSession};
use crate::models::{PlatformSettings, UpdatePlatformSettingsRequest};

pub async fn get(
    State(state): State<Arc<AppState>>,
    session: AuthSession,
) -> AppResult<Json<PlatformSettings>> {
    if !is_admin(&session.user) {
        return Err(AppError::Forbidden);
    }

    let settings = sqlx::query_as::<_, PlatformSettings>(
        "SELECT * FROM platform_settings WHERE id = 1",
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::Internal("platform settings missing".into()))?;

    Ok(Json(settings))
}

pub async fn update(
    State(state): State<Arc<AppState>>,
    session: AuthSession,
    Json(body): Json<UpdatePlatformSettingsRequest>,
) -> AppResult<Json<PlatformSettings>> {
    if !is_admin(&session.user) {
        return Err(AppError::Forbidden);
    }

    let role = body.default_user_role.trim();
    if !VALID_DEFAULT_ROLES.contains(&role) {
        return Err(AppError::BadRequest(
            "default_user_role must be regular or proponent".into(),
        ));
    }

    let now = Utc::now();
    sqlx::query(
        "UPDATE platform_settings SET default_user_role = ?, updated_at = ? WHERE id = 1",
    )
    .bind(role)
    .bind(now)
    .execute(&state.pool)
    .await?;

    let settings = sqlx::query_as::<_, PlatformSettings>(
        "SELECT * FROM platform_settings WHERE id = 1",
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(settings))
}
