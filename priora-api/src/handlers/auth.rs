use std::sync::Arc;

use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect},
    Json,
};
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthUrl, AuthorizationCode, ClientId,
    ClientSecret, CsrfToken, RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use serde::Deserialize;

use crate::auth::{
    create_token, dev_login_user, find_or_create_oauth_user, get_user_by_id, is_admin,
    resolve_user_ref,
};
use crate::db::fetch_user_public;
use crate::error::{AppError, AppResult};
use crate::handlers::{AppState, AuthSession, OptionalAuthSession};
use crate::models::{AuthResponse, DevLoginRequest, MeResponse};

#[derive(Deserialize)]
pub struct CallbackQuery {
    code: Option<String>,
    error: Option<String>,
}

#[derive(Deserialize)]
pub struct ImpersonateQuery {
    priora_as: String,
}

fn google_client(config: &crate::config::Config) -> AppResult<BasicClient> {
    let client_id = config
        .google_client_id
        .as_ref()
        .ok_or_else(|| AppError::BadRequest("Google OAuth not configured".into()))?;
    let client_secret = config
        .google_client_secret
        .as_ref()
        .ok_or_else(|| AppError::BadRequest("Google OAuth not configured".into()))?;
    let redirect_uri = config
        .google_redirect_uri
        .as_ref()
        .ok_or_else(|| AppError::BadRequest("Google OAuth not configured".into()))?;

    Ok(BasicClient::new(
        ClientId::new(client_id.clone()),
        Some(ClientSecret::new(client_secret.clone())),
        AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".into())
            .map_err(|e| AppError::Internal(e.to_string()))?,
        Some(
            TokenUrl::new("https://oauth2.googleapis.com/token".into())
                .map_err(|e| AppError::Internal(e.to_string()))?,
        ),
    )
    .set_redirect_uri(
        RedirectUrl::new(redirect_uri.clone()).map_err(|e| AppError::Internal(e.to_string()))?,
    ))
}

pub async fn google_login(State(state): State<Arc<AppState>>) -> AppResult<impl IntoResponse> {
    if !state.config.google_oauth_enabled() {
        return Err(AppError::BadRequest(
            "Google OAuth not configured. Use dev login.".into(),
        ));
    }

    let client = google_client(&state.config)?;
    let (auth_url, _csrf) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("openid".into()))
        .add_scope(Scope::new("email".into()))
        .add_scope(Scope::new("profile".into()))
        .url();

    Ok(Redirect::to(auth_url.as_str()))
}

pub async fn google_callback(
    State(state): State<Arc<AppState>>,
    Query(params): Query<CallbackQuery>,
) -> AppResult<impl IntoResponse> {
    if params.error.is_some() {
        return Err(AppError::BadRequest("OAuth denied".into()));
    }
    let code = params
        .code
        .ok_or_else(|| AppError::BadRequest("missing code".into()))?;

    let client = google_client(&state.config)?;
    let token = client
        .exchange_code(AuthorizationCode::new(code))
        .request_async(async_http_client)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let access_token = token.access_token().secret();
    let user_info: serde_json::Value = reqwest::Client::new()
        .get("https://www.googleapis.com/oauth2/v3/userinfo")
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .json()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let google_sub = user_info["sub"]
        .as_str()
        .ok_or_else(|| AppError::Internal("missing sub".into()))?;
    let email = user_info["email"]
        .as_str()
        .unwrap_or("unknown@google.com");
    let name = user_info["name"].as_str().unwrap_or("Usuario");
    let picture = user_info["picture"].as_str();

    let user = find_or_create_oauth_user(
        &state.pool,
        google_sub,
        email,
        name,
        picture,
    )
    .await?;
    let jwt = create_token(&user.id, None, &state.config)?;
    let redirect = format!("{}/auth/callback?token={}", state.config.frontend_url, jwt);
    Ok(Redirect::to(&redirect))
}

pub async fn dev_login(
    State(state): State<Arc<AppState>>,
    Json(body): Json<DevLoginRequest>,
) -> AppResult<Json<AuthResponse>> {
    if !state.config.dev_auth {
        return Err(AppError::Forbidden);
    }

    let role = body.role.as_deref();
    let user = dev_login_user(&state.pool, &body.email, &body.name, role).await?;
    let token = create_token(&user.id, None, &state.config)?;
    Ok(Json(AuthResponse {
        token,
        user,
        impersonator: None,
    }))
}

pub async fn impersonate(
    State(state): State<Arc<AppState>>,
    auth: OptionalAuthSession,
    Query(query): Query<ImpersonateQuery>,
) -> AppResult<Json<AuthResponse>> {
    let target = resolve_user_ref(&state.pool, query.priora_as.trim()).await?;

    let impersonator_id = match (&auth.session, state.config.dev_impersonation) {
        (Some(session), _) if is_admin(&session.user) => Some(session.user.id.clone()),
        (_, true) => None,
        _ => return Err(AppError::Forbidden),
    };

    let impersonator_id = impersonator_id.as_deref();
    let token = create_token(&target.id, impersonator_id, &state.config)?;

    let impersonator = if let Some(id) = impersonator_id {
        Some(fetch_user_public(&state.pool, id).await?)
    } else {
        None
    };

    Ok(Json(AuthResponse {
        token,
        user: target,
        impersonator,
    }))
}

pub async fn stop_impersonate(
    State(state): State<Arc<AppState>>,
    session: AuthSession,
) -> AppResult<Json<AuthResponse>> {
    let impersonator_id = session
        .impersonator_id
        .as_deref()
        .ok_or(AppError::BadRequest("not impersonating".into()))?;

    let admin = get_user_by_id(&state.pool, impersonator_id).await?;
    let token = create_token(&admin.id, None, &state.config)?;

    Ok(Json(AuthResponse {
        token,
        user: admin,
        impersonator: None,
    }))
}

pub async fn me(
    State(state): State<Arc<AppState>>,
    session: AuthSession,
) -> AppResult<Json<MeResponse>> {
    let impersonator = if let Some(id) = session.impersonator_id.as_deref() {
        Some(fetch_user_public(&state.pool, id).await?)
    } else {
        None
    };

    Ok(Json(MeResponse {
        user: session.user,
        impersonator,
    }))
}

pub async fn logout() -> impl IntoResponse {
    (axum::http::StatusCode::OK, Json(serde_json::json!({ "ok": true })))
}
