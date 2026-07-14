use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::Config;
use crate::error::{AppError, AppResult};
use crate::models::User;
use sqlx::SqlitePool;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub impersonator_id: Option<String>,
}

pub fn create_token(
    user_id: &str,
    impersonator_id: Option<&str>,
    config: &Config,
) -> AppResult<String> {
    let exp = (Utc::now() + Duration::days(7)).timestamp();
    let claims = Claims {
        sub: user_id.to_string(),
        exp,
        impersonator_id: impersonator_id.map(str::to_string),
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(e.to_string()))
}

pub fn verify_token(token: &str, config: &Config) -> AppResult<Claims> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|_| AppError::Unauthorized)
}

pub async fn get_user_by_id(pool: &SqlitePool, id: &str) -> AppResult<User> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)
}

pub async fn get_user_by_email(pool: &SqlitePool, email: &str) -> AppResult<User> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = ?")
        .bind(email)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)
}

pub async fn resolve_user_ref(pool: &SqlitePool, reference: &str) -> AppResult<User> {
    if reference.contains('@') {
        get_user_by_email(pool, reference).await
    } else {
        get_user_by_id(pool, reference).await
    }
}

/// Roles allowed as the platform default for newly registered users.
pub const VALID_DEFAULT_ROLES: &[&str] = &["regular", "proponent"];

pub async fn get_default_user_role(pool: &SqlitePool) -> AppResult<String> {
    let role = sqlx::query_scalar::<_, String>(
        "SELECT default_user_role FROM platform_settings WHERE id = 1",
    )
    .fetch_optional(pool)
    .await?;

    Ok(match role {
        Some(r) if VALID_DEFAULT_ROLES.contains(&r.as_str()) => r,
        _ => "proponent".to_string(),
    })
}

pub async fn find_or_create_oauth_user(
    pool: &SqlitePool,
    google_sub: &str,
    email: &str,
    name: &str,
    picture_url: Option<&str>,
) -> AppResult<User> {
    if let Some(user) = sqlx::query_as::<_, User>("SELECT * FROM users WHERE google_sub = ?")
        .bind(google_sub)
        .fetch_optional(pool)
        .await?
    {
        return Ok(user);
    }

    let role = get_default_user_role(pool).await?;
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();
    sqlx::query(
        "INSERT INTO users (id, google_sub, email, name, picture_url, role, profile_complete, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, 0, ?, ?)",
    )
    .bind(&id)
    .bind(google_sub)
    .bind(email)
    .bind(name)
    .bind(picture_url)
    .bind(&role)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;

    get_user_by_id(pool, &id).await
}

pub async fn dev_login_user(
    pool: &SqlitePool,
    email: &str,
    name: &str,
    role: Option<&str>,
) -> AppResult<User> {
    let google_sub = format!("dev:{email}");
    let role = match role {
        Some(r) => r.to_string(),
        None => get_default_user_role(pool).await?,
    };

    if let Some(user) = sqlx::query_as::<_, User>("SELECT * FROM users WHERE google_sub = ?")
        .bind(&google_sub)
        .fetch_optional(pool)
        .await?
    {
        if user.role != role {
            sqlx::query("UPDATE users SET role = ?, updated_at = ? WHERE id = ?")
                .bind(&role)
                .bind(Utc::now())
                .bind(&user.id)
                .execute(pool)
                .await?;
            return get_user_by_id(pool, &user.id).await;
        }
        return Ok(user);
    }

    let id = Uuid::new_v4().to_string();
    let now = Utc::now();
    sqlx::query(
        "INSERT INTO users (id, google_sub, email, name, role, profile_complete, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, 0, ?, ?)",
    )
    .bind(&id)
    .bind(&google_sub)
    .bind(email)
    .bind(name)
    .bind(&role)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;

    get_user_by_id(pool, &id).await
}

pub fn is_admin(user: &User) -> bool {
    user.role == "admin"
}

pub fn ensure_profile(user: &User) -> AppResult<()> {
    if user.profile_complete {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}
