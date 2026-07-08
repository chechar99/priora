use std::env;

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub host: String,
    pub port: u16,
    pub frontend_url: String,
    pub dev_auth: bool,
    pub dev_impersonation: bool,
    pub impersonate_query_key: String,
    pub seed_demo_data: bool,
    pub google_client_id: Option<String>,
    pub google_client_secret: Option<String>,
    pub google_redirect_uri: Option<String>,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:priora.db?mode=rwc".into()),
            jwt_secret: env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret-change-me".into()),
            host: env::var("HOST").unwrap_or_else(|_| "127.0.0.1".into()),
            port: env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3000),
            frontend_url: env::var("FRONTEND_URL")
                .unwrap_or_else(|_| "http://localhost:5173".into()),
            dev_auth: env::var("DEV_AUTH")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(true),
            dev_impersonation: env::var("DEV_IMPERSONATION")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false),
            impersonate_query_key: env::var("IMPERSONATE_QUERY_KEY")
                .unwrap_or_else(|_| "priora_as".into()),
            seed_demo_data: env::var("SEED_DEMO_DATA")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(true),
            google_client_id: env::var("GOOGLE_CLIENT_ID").ok().filter(|s| !s.is_empty()),
            google_client_secret: env::var("GOOGLE_CLIENT_SECRET")
                .ok()
                .filter(|s| !s.is_empty()),
            google_redirect_uri: env::var("GOOGLE_REDIRECT_URI").ok().filter(|s| !s.is_empty()),
        }
    }

    pub fn google_oauth_enabled(&self) -> bool {
        self.google_client_id.is_some()
            && self.google_client_secret.is_some()
            && self.google_redirect_uri.is_some()
    }
}
