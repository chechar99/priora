use std::sync::Arc;

use axum::{extract::State, Json};
use chrono::Utc;

use crate::error::{AppError, AppResult};
use crate::handlers::{AppState, AuthSession};
use crate::models::{UpdateProfileRequest, User};

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

    let updated = crate::auth::get_user_by_id(&state.pool, &session.user.id).await?;
    Ok(Json(updated))
}
