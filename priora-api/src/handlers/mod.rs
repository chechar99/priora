mod auth;
mod categories;
mod comments;
mod activity;
mod membership;
mod namespaces;
mod proposals;
mod rankings;
mod stats;
mod uploads;
mod users;

use std::sync::Arc;

use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts},
    routing::{delete, get, patch, post},
    Router,
};
use axum::http::Method;
use sqlx::SqlitePool;
use tower_http::cors::{AllowHeaders, AllowMethods, CorsLayer};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

use crate::auth::{get_user_by_id, verify_token};
use crate::config::Config;
use crate::error::AppError;
use crate::models::User;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub config: Config,
}

pub struct AuthSession {
    pub user: User,
    pub impersonator_id: Option<String>,
}

impl FromRequestParts<Arc<AppState>> for AuthSession {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let token = extract_token(parts).ok_or(AppError::Unauthorized)?;
        let claims = verify_token(&token, &state.config)?;
        let user = get_user_by_id(&state.pool, &claims.sub).await?;
        Ok(AuthSession {
            user,
            impersonator_id: claims.impersonator_id,
        })
    }
}

pub struct OptionalAuthSession {
    pub session: Option<AuthSession>,
}

impl FromRequestParts<Arc<AppState>> for OptionalAuthSession {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let token = match extract_token(parts) {
            Some(token) => token,
            None => return Ok(OptionalAuthSession { session: None }),
        };

        let claims = match verify_token(&token, &state.config) {
            Ok(claims) => claims,
            Err(_) => return Ok(OptionalAuthSession { session: None }),
        };

        let user = match get_user_by_id(&state.pool, &claims.sub).await {
            Ok(user) => user,
            Err(_) => return Ok(OptionalAuthSession { session: None }),
        };

        Ok(OptionalAuthSession {
            session: Some(AuthSession {
                user,
                impersonator_id: claims.impersonator_id,
            }),
        })
    }
}

fn extract_token(parts: &Parts) -> Option<String> {
    parts
        .headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::to_string)
        .or_else(|| {
            parts
                .headers
                .get("Cookie")
                .and_then(|v| v.to_str().ok())
                .and_then(|cookies| {
                    cookies
                        .split(';')
                        .find_map(|c| c.trim().strip_prefix("token="))
                        .map(|t| t.to_string())
                })
        })
}

pub fn build_router(state: Arc<AppState>) -> Router {
    let scoped = Router::new()
        .route("/proposals", get(proposals::list).post(proposals::create))
        .route(
            "/proposals/{id}",
            get(proposals::get_one).patch(proposals::update),
        )
        .route("/proposals/{id}/status", patch(proposals::update_status))
        .route("/proposals/{id}/tracker", patch(proposals::update_tracker))
        .route(
            "/proposals/{id}/comments",
            get(comments::list).post(comments::create),
        )
        .route("/comments/{id}", delete(comments::delete_comment))
        .route("/rankings/me", get(rankings::get_my).put(rankings::save_my))
        .route("/stats", get(stats::dashboard))
        .route("/activity/me", get(activity::my_activity))
        .route("/membership/me", get(membership::me))
        .route("/membership/request", post(membership::request))
        .route("/membership/accept-invite", post(membership::redeem_invite))
        .route("/invite", get(namespaces::get_invite).post(namespaces::regenerate_invite))
        .route("/members", get(membership::list))
        .route("/members/{user_id}", patch(membership::update));

    let api = Router::new()
        .route("/auth/google", get(auth::google_login))
        .route("/auth/google/callback", get(auth::google_callback))
        .route("/auth/dev-login", post(auth::dev_login))
        .route("/auth/impersonate", get(auth::impersonate))
        .route("/auth/stop-impersonate", post(auth::stop_impersonate))
        .route("/auth/me", get(auth::me))
        .route("/auth/logout", post(auth::logout))
        .route("/users/me", get(users::get_me).patch(users::update_me))
        .route("/users", get(users::list))
        .route("/users/{id}/role", patch(users::update_role))
        .route("/categories", get(categories::list))
        .route("/namespaces", get(namespaces::list).post(namespaces::create))
        .route(
            "/namespaces/{slug}",
            get(namespaces::get_one).patch(namespaces::update),
        )
        .route("/uploads/image", post(uploads::upload_image))
        .nest("/{namespace}", scoped)
        .with_state(state.clone());

    Router::new()
        .route("/api/health", get(|| async { "ok" }))
        .nest("/api", api)
        .nest_service("/uploads", ServeDir::new("uploads"))
        .layer(
            CorsLayer::new()
                .allow_origin(
                    state
                        .config
                        .frontend_url
                        .parse::<axum::http::HeaderValue>()
                        .unwrap(),
                )
                .allow_methods(AllowMethods::list([
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::PATCH,
                    Method::DELETE,
                ]))
                .allow_headers(AllowHeaders::list([
                    axum::http::header::AUTHORIZATION,
                    axum::http::header::CONTENT_TYPE,
                ]))
                .allow_credentials(true),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
