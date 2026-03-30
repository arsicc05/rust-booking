use axum::body::Body;
use axum::extract::{Path, Request, State};
use axum::http::{HeaderMap, Method};
use axum::response::{IntoResponse, Response};
use axum::Json;
use shared::errors::AppError;

use crate::AppState;

pub async fn proxy_auth(
    State(state): State<AppState>,
    Path(path): Path<String>,
    req: Request<Body>,
) -> Result<Response, AppError> {
    let url = format!("{}/auth/{}", state.services.auth, path);
    proxy_request(
        &state.http_client,
        req.method().clone(),
        &url,
        req.headers().clone(),
        req,
    )
    .await
}

pub async fn proxy_user(
    State(state): State<AppState>,
    Path(path): Path<String>,
    req: Request<Body>,
) -> Result<Response, AppError> {
    let url = format!("{}/users/{}", state.services.user, path);
    proxy_request(
        &state.http_client,
        req.method().clone(),
        &url,
        req.headers().clone(),
        req,
    )
    .await
}

pub async fn proxy_appointment(
    State(state): State<AppState>,
    Path(path): Path<String>,
    req: Request<Body>,
) -> Result<Response, AppError> {
    let url = format!("{}/appointments/{}", state.services.appointment, path);
    proxy_request(
        &state.http_client,
        req.method().clone(),
        &url,
        req.headers().clone(),
        req,
    )
    .await
}

pub async fn proxy_notification(
    State(state): State<AppState>,
    Path(path): Path<String>,
    req: Request<Body>,
) -> Result<Response, AppError> {
    let url = format!("{}/notifications/{}", state.services.notification, path);
    proxy_request(
        &state.http_client,
        req.method().clone(),
        &url,
        req.headers().clone(),
        req,
    )
    .await
}

pub async fn composed_my_appointments(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    let auth_header = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();

    let appointments_resp = state
        .http_client
        .get(format!("{}/appointments/my", state.services.appointment))
        .header("Authorization", auth_header)
        .send()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .json::<serde_json::Value>()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let notifications_resp = state
        .http_client
        .get(format!("{}/notifications/my", state.services.notification))
        .header("Authorization", auth_header)
        .send()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .json::<serde_json::Value>()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let result = serde_json::json!({
        "success": true,
        "data": {
            "appointments": appointments_resp.get("data"),
            "notifications": notifications_resp.get("data"),
        }
    });

    Ok(Json(result))
}

async fn proxy_request(
    client: &reqwest::Client,
    method: Method,
    url: &str,
    headers: HeaderMap,
    req: Request<Body>,
) -> Result<Response, AppError> {
    let body_bytes = axum::body::to_bytes(req.into_body(), usize::MAX)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let mut proxy_req = client.request(method, url);

    if let Some(ct) = headers.get("content-type") {
        proxy_req = proxy_req.header("content-type", ct);
    }
    if let Some(auth) = headers.get("authorization") {
        proxy_req = proxy_req.header("authorization", auth);
    }

    if !body_bytes.is_empty() {
        proxy_req = proxy_req.body(body_bytes);
    }

    let resp = proxy_req
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Upstream error: {}", e)))?;

    let status = resp.status();
    let resp_headers = resp.headers().clone();
    let resp_body = resp
        .bytes()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let mut response = Response::builder().status(status);
    if let Some(ct) = resp_headers.get("content-type") {
        response = response.header("content-type", ct);
    }

    response
        .body(Body::from(resp_body))
        .map_err(|e| AppError::Internal(e.to_string()))
}
