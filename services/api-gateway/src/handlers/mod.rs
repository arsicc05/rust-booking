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
    let is_register = path == "register";

    let base = format!("{}/auth/{}", state.services.auth, path);
    let url = append_query_string(&base, req.uri());

    if is_register {
        let headers = req.headers().clone();
        let body_bytes = axum::body::to_bytes(req.into_body(), usize::MAX)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let reg_body: serde_json::Value =
            serde_json::from_slice(&body_bytes).map_err(|e| AppError::BadRequest(e.to_string()))?;
        let role = reg_body
            .get("role")
            .and_then(|v| v.as_str())
            .unwrap_or("user")
            .to_string();

        let mut proxy_req = state.http_client.request(Method::POST, &url);
        if let Some(ct) = headers.get("content-type") {
            proxy_req = proxy_req.header("content-type", ct);
        }
        proxy_req = proxy_req.body(body_bytes.clone());

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

        if status.is_success() {
            if let Ok(auth_resp) = serde_json::from_slice::<serde_json::Value>(&resp_body) {
                if let Some(user_id) = auth_resp.pointer("/data/user_id").and_then(|v| v.as_str()) {
                    let service_type = if role == "provider" {
                        Some("general")
                    } else {
                        None
                    };

                    let profile_body = serde_json::json!({
                        "user_id": user_id,
                        "role": role,
                        "service_type": service_type,
                    });

                    let profile_url = format!("{}/users", state.services.user);
                    let _ = state
                        .http_client
                        .post(&profile_url)
                        .header("content-type", "application/json")
                        .json(&profile_body)
                        .send()
                        .await;

                    tracing::info!("Created profile for user {} (role={})", user_id, role);
                }
            }
        }

        let mut response = Response::builder().status(status);
        if let Some(ct) = resp_headers.get("content-type") {
            response = response.header("content-type", ct);
        }
        return response
            .body(Body::from(resp_body))
            .map_err(|e| AppError::Internal(e.to_string()));
    }

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
    let base = format!("{}/users/{}", state.services.user, path);
    let url = append_query_string(&base, req.uri());
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
    let base = format!("{}/appointments/{}", state.services.appointment, path);
    let url = append_query_string(&base, req.uri());
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
    let base = format!("{}/notifications/{}", state.services.notification, path);
    let url = append_query_string(&base, req.uri());
    proxy_request(
        &state.http_client,
        req.method().clone(),
        &url,
        req.headers().clone(),
        req,
    )
    .await
}

pub async fn proxy_uploads(
    State(state): State<AppState>,
    Path(path): Path<String>,
    req: Request<Body>,
) -> Result<Response, AppError> {
    let base = format!("{}/uploads/{}", state.services.user, path);
    let url = append_query_string(&base, req.uri());
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

fn append_query_string(base_url: &str, uri: &axum::http::Uri) -> String {
    match uri.query() {
        Some(qs) => format!("{}?{}", base_url, qs),
        None => base_url.to_string(),
    }
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
    if let Some(user_id) = headers.get("X-User-Id") {
        proxy_req = proxy_req.header("X-User-Id", user_id);
    }
    if let Some(role) = headers.get("X-User-Role") {
        proxy_req = proxy_req.header("X-User-Role", role);
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
