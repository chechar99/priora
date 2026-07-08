use std::sync::Arc;

use axum::extract::{Path, State};
use axum::Json;

use crate::db::fetch_namespace_by_slug;
use crate::error::AppResult;
use crate::handlers::AppState;
use crate::models::Namespace;

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
