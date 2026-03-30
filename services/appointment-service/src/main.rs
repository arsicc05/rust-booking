mod dto;
mod handlers;
mod models;
mod repository;
mod service;

use axum::{
    routing::{get, post},
    Router,
};
use shared::config::{DatabaseConfig, NatsConfig};
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::EnvFilter;

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub nats: Option<async_nats::Client>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env().add_directive("appointment_service=debug".parse()?),
        )
        .init();

    let db_config = DatabaseConfig::from_env();
    let nats_config = NatsConfig::from_env();

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&db_config.url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let nats = match async_nats::connect(&nats_config.url).await {
        Ok(client) => {
            tracing::info!("Connected to NATS at {}", nats_config.url);
            Some(client)
        }
        Err(e) => {
            tracing::warn!("Failed to connect to NATS: {}. Running without events.", e);
            None
        }
    };

    let state = AppState { db: pool, nats };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/appointments/slots", post(handlers::create_slots))
        .route("/appointments/slots", get(handlers::get_available_slots))
        .route("/appointments/book", post(handlers::book_appointment))
        .route(
            "/appointments/:id/cancel",
            post(handlers::cancel_appointment),
        )
        .route("/appointments/my", get(handlers::my_appointments))
        .route("/health", get(health))
        .layer(cors)
        .with_state(state);

    let addr = "0.0.0.0:3003";
    tracing::info!("Appointment Service listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health() -> &'static str {
    "OK"
}
