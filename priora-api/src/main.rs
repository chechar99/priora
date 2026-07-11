use std::net::SocketAddr;
use std::sync::Arc;

use sqlx::sqlite::SqlitePoolOptions;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use priora_api::config::Config;
use priora_api::db::seed_demo_data;
use priora_api::handlers::{build_router, AppState};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "priora_api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env();
    std::fs::create_dir_all("uploads").ok();

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;
    if config.seed_demo_data {
        seed_demo_data(&pool).await?;
    }

    let state = Arc::new(AppState {
        pool,
        config: config.clone(),
    });

    let app = build_router(state);
    let addr = SocketAddr::from((config.host.parse::<std::net::IpAddr>()?, config.port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("Priora API listening on http://{addr}");
    if config.dev_auth {
        tracing::info!("Dev auth enabled — POST /api/auth/dev-login");
    }
    if config.dev_impersonation {
        tracing::info!(
            "Dev impersonation enabled — GET /api/auth/impersonate?priora_as=<email>"
        );
    }

    axum::serve(listener, app).await?;

    Ok(())
}
