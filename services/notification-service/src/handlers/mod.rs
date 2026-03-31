use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::Json;
use shared::errors::AppError;
use shared::models::ApiResponse;
use uuid::Uuid;

use crate::dto::*;
use crate::service;
use crate::AppState;

pub async fn get_notifications(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<NotificationsQuery>,
) -> Result<Json<ApiResponse<Vec<NotificationResponse>>>, AppError> {
    let user_id = query.user_id.or_else(|| {
        headers
            .get("X-User-Id")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| Uuid::parse_str(v).ok())
    }).ok_or_else(|| AppError::BadRequest("Missing user_id".into()))?;

    let final_query = NotificationsQuery {
        user_id: Some(user_id),
    };
    let notifications = service::get_notifications(&state.db, final_query).await?;
    Ok(Json(ApiResponse::success(notifications)))
}

pub async fn get_qr_code(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    let qr = service::get_qr_code(&state.db, &id).await?;
    Ok(Json(ApiResponse::success(qr)))
}
