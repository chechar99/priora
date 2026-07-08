use std::sync::Arc;

use axum::extract::State;
use axum::Json;

use crate::error::AppResult;
use crate::handlers::AppState;
use crate::models::Category;

pub async fn list(State(state): State<Arc<AppState>>) -> AppResult<Json<Vec<Category>>> {
    let categories = sqlx::query_as::<_, Category>(
        "SELECT id, name FROM categories ORDER BY name ASC",
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(categories))
}
