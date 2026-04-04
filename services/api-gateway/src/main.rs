mod handlers;
mod middleware;
mod models;

use axum::{middleware as axum_middleware, routing::any, routing::get, Router};
use models::ServiceUrls;
use shared::config::JwtConfig;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::EnvFilter;

#[derive(Clone)]
pub struct AppState {
    pub jwt_config: JwtConfig,
    pub services: ServiceUrls,
    pub http_client: reqwest::Client,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("api_gateway=debug".parse()?))
        .init();

    let jwt_config = JwtConfig::from_env();
    let services = ServiceUrls::from_env();

    tracing::info!(
        "Service URLs: auth={}, user={}, appointment={}, notification={}",
        services.auth,
        services.user,
        services.appointment,
        services.notification
    );

    let state = AppState {
        jwt_config,
        services,
        http_client: reqwest::Client::new(),
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/auth/{*path}", any(handlers::proxy_auth))
        .route("/api/users/{*path}", any(handlers::proxy_user))
        .route(
            "/api/appointments/{*path}",
            any(handlers::proxy_appointment),
        )
        .route(
            "/api/appointments/my/detailed",
            get(handlers::composed_my_appointments),
        )
        .route(
            "/api/notifications/{*path}",
            any(handlers::proxy_notification),
        )
        .route("/uploads/{*path}", any(handlers::proxy_uploads))
        .route("/health", get(health))
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::auth_middleware,
        ))
        .layer(cors)
        .with_state(state);

    let addr = "0.0.0.0:4000";
    tracing::info!("API Gateway listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health() -> &'static str {
    "OK"
}
