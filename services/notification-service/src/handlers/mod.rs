use axum::extract::{Path, Query, State};
use axum::Json;
use shared::errors::AppError;
use shared::models::ApiResponse;

use crate::dto::*;
use crate::service;
use crate::AppState;

pub async fn get_notifications(
    State(state): State<AppState>,
    Query(query): Query<NotificationsQuery>,
) -> Result<Json<ApiResponse<Vec<NotificationResponse>>>, AppError> {
    let notifications = service::get_notifications(&state.db, query).await?;
    Ok(Json(ApiResponse::success(notifications)))
}

pub async fn get_qr_code(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    let qr = service::get_qr_code(&state.db, &id).await?;
    Ok(Json(ApiResponse::success(qr)))
}
