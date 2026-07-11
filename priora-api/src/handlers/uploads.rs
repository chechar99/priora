use std::path::Path;
use std::sync::Arc;

use axum::extract::{Multipart, State};
use axum::Json;
use uuid::Uuid;

use crate::auth::ensure_profile;
use crate::error::{AppError, AppResult};
use crate::handlers::{AppState, AuthSession};

const MAX_LOGO_BYTES: usize = 2 * 1024 * 1024;

pub async fn upload_logo(
    State(_state): State<Arc<AppState>>,
    session: AuthSession,
    mut multipart: Multipart,
) -> AppResult<Json<serde_json::Value>> {
    ensure_profile(&session.user)?;

    let field = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("invalid multipart: {e}")))?
        .ok_or_else(|| AppError::BadRequest("file required".into()))?;

    let content_type = field
        .content_type()
        .map(|m| m.to_string())
        .unwrap_or_default();

    let ext = match content_type.as_str() {
        "image/jpeg" | "image/jpg" => "jpg",
        "image/png" => "png",
        "image/webp" => "webp",
        _ => {
            return Err(AppError::BadRequest(
                "logo must be JPEG, PNG, or WebP".into(),
            ));
        }
    };

    let data = field
        .bytes()
        .await
        .map_err(|e| AppError::BadRequest(format!("failed to read file: {e}")))?;

    if data.is_empty() {
        return Err(AppError::BadRequest("empty file".into()));
    }
    if data.len() > MAX_LOGO_BYTES {
        return Err(AppError::BadRequest("logo must be at most 2 MB".into()));
    }

    let filename = format!("{}.{}", Uuid::new_v4(), ext);
    let path = Path::new("uploads").join(&filename);
    tokio::fs::write(&path, &data)
        .await
        .map_err(|e| AppError::Internal(format!("failed to save upload: {e}")))?;

    let url = format!("/uploads/{filename}");
    Ok(Json(serde_json::json!({ "url": url })))
}
