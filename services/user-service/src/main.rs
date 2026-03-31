mod dto;
mod handlers;
mod models;
mod repository;
mod service;

use axum::{
    routing::{get, post, put},
    Router,
};
use shared::config::DatabaseConfig;
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::EnvFilter;

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("user_service=debug".parse()?))
        .init();

    let db_config = DatabaseConfig::from_env();

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&db_config.url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let state = AppState { db: pool };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/users/{user_id}", get(handlers::get_profile))
        .route("/users", post(handlers::create_profile))
        .route("/users/{user_id}", put(handlers::update_profile))
        .route("/users/providers", get(handlers::list_providers))
        .route("/users/providers/{user_id}", get(handlers::get_provider))
        .route("/users/{user_id}/avatar", post(handlers::upload_avatar))
        .route("/health", get(health))
        .layer(cors)
        .with_state(state);

    let addr = "0.0.0.0:3002";
    tracing::info!("User Service listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health() -> &'static str {
    "OK"
}
