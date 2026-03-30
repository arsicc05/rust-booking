mod dto;
mod events;
mod handlers;
mod models;
mod repository;
mod service;

use axum::{routing::get, Router};
use futures_util::StreamExt;
use shared::config::{MongoConfig, NatsConfig};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::EnvFilter;

use crate::models::AppointmentEvent;

#[derive(Clone)]
pub struct AppState {
    pub db: mongodb::Database,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env().add_directive("notification_service=debug".parse()?),
        )
        .init();

    let mongo_config = MongoConfig::from_env();
    let nats_config = NatsConfig::from_env();

    let mongo_client = mongodb::Client::with_uri_str(&mongo_config.url).await?;
    let db = mongo_client.database(&mongo_config.database);
    tracing::info!("Connected to MongoDB");

    let state = AppState { db: db.clone() };

    let db_for_nats = db.clone();
    tokio::spawn(async move {
        if let Err(e) = run_nats_subscriber(&nats_config.url, &db_for_nats).await {
            tracing::error!("NATS subscriber error: {}", e);
        }
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/notifications/my", get(handlers::get_notifications))
        .route("/notifications/:id/qr", get(handlers::get_qr_code))
        .route("/health", get(health))
        .layer(cors)
        .with_state(state);

    let addr = "0.0.0.0:3004";
    tracing::info!("Notification Service listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn run_nats_subscriber(nats_url: &str, db: &mongodb::Database) -> anyhow::Result<()> {
    let client = match async_nats::connect(nats_url).await {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(
                "Failed to connect to NATS: {}. Notifications won't receive events.",
                e
            );
            return Ok(());
        }
    };

    tracing::info!("NATS subscriber connected");

    let mut sub_created = client.subscribe("appointments.created").await?;
    let mut sub_cancelled = client.subscribe("appointments.cancelled").await?;

    loop {
        tokio::select! {
            Some(msg) = sub_created.next() => {
                match serde_json::from_slice::<AppointmentEvent>(&msg.payload) {
                    Ok(event) => {
                        if let Err(e) = events::handle_appointment_created(db, &event).await {
                            tracing::error!("Failed to handle AppointmentCreated: {}", e);
                        }
                    }
                    Err(e) => tracing::error!("Failed to deserialize event: {}", e),
                }
            }
            Some(msg) = sub_cancelled.next() => {
                match serde_json::from_slice::<AppointmentEvent>(&msg.payload) {
                    Ok(event) => {
                        if let Err(e) = events::handle_appointment_cancelled(db, &event).await {
                            tracing::error!("Failed to handle AppointmentCancelled: {}", e);
                        }
                    }
                    Err(e) => tracing::error!("Failed to deserialize event: {}", e),
                }
            }
        }
    }
}

async fn health() -> &'static str {
    "OK"
}
