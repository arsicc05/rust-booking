use axum::{extract::State, http::HeaderMap, Json};
use shared::auth;
use shared::errors::AppError;
use shared::models::ApiResponse;

use crate::dto::*;
use crate::service;
use crate::AppState;

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<ApiResponse<AuthResponse>>, AppError> {
    let result = service::register(&state.db, &state.jwt_config, payload).await?;
    Ok(Json(ApiResponse::success(result)))
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<ApiResponse<AuthResponse>>, AppError> {
    let result = service::login(&state.db, &state.jwt_config, &payload.email, &payload.password).await?;
    Ok(Json(ApiResponse::success(result)))
}

pub async fn refresh(
    State(state): State<AppState>,
    Json(payload): Json<RefreshRequest>,
) -> Result<Json<ApiResponse<AuthResponse>>, AppError> {
    let result = service::refresh(&state.db, &state.jwt_config, &payload.refresh_token).await?;
    Ok(Json(ApiResponse::success(result)))
}

pub async fn validate(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<ValidateResponse>>, AppError> {
    let auth_header = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("Missing Authorization header".into()))?;

    let token = auth::extract_token_from_header(auth_header)
        .ok_or_else(|| AppError::Unauthorized("Invalid authorization format".into()))?;

    let result = service::validate_token(&state.jwt_config, token);
    Ok(Json(ApiResponse::success(result)))
}
