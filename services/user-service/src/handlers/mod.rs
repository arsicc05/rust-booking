use axum::extract::{Multipart, Path, Query, State};
use axum::Json;
use shared::errors::AppError;
use shared::models::ApiResponse;
use uuid::Uuid;

use crate::dto::*;
use crate::models::Profile;
use crate::service;
use crate::AppState;

pub async fn get_profile(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<ApiResponse<Profile>>, AppError> {
    let profile = service::get_profile(&state.db, user_id).await?;
    Ok(Json(ApiResponse::success(profile)))
}

pub async fn create_profile(
    State(state): State<AppState>,
    Json(payload): Json<CreateProfileRequest>,
) -> Result<Json<ApiResponse<Profile>>, AppError> {
    let profile = service::create_profile(&state.db, payload).await?;
    Ok(Json(ApiResponse::success(profile)))
}

pub async fn update_profile(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Json(payload): Json<UpdateProfileRequest>,
) -> Result<Json<ApiResponse<Profile>>, AppError> {
    let profile = service::update_profile(&state.db, user_id, payload).await?;
    Ok(Json(ApiResponse::success(profile)))
}

pub async fn list_providers(
    State(state): State<AppState>,
    Query(query): Query<ProviderQuery>,
) -> Result<Json<ApiResponse<Vec<Profile>>>, AppError> {
    let profiles = service::list_providers(
        &state.db,
        query.service_type.as_deref(),
        query.search.as_deref(),
    )
    .await?;
    Ok(Json(ApiResponse::success(profiles)))
}

pub async fn get_provider(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<ApiResponse<Profile>>, AppError> {
    let profile = service::get_provider(&state.db, user_id).await?;
    Ok(Json(ApiResponse::success(profile)))
}

pub async fn upload_avatar(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<Json<ApiResponse<Profile>>, AppError> {
    let mut file_data: Option<Vec<u8>> = None;
    let mut file_name: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?
    {
        if field.name() == Some("avatar") {
            file_name = field.file_name().map(|s| s.to_string());
            file_data = Some(
                field
                    .bytes()
                    .await
                    .map_err(|e| AppError::BadRequest(e.to_string()))?
                    .to_vec(),
            );
            break;
        }
    }

    let data = file_data.ok_or_else(|| AppError::BadRequest("No avatar file provided".into()))?;
    let name = file_name.unwrap_or_else(|| "avatar.png".to_string());

    let profile = service::upload_avatar(&state.db, user_id, &name, &data).await?;
    Ok(Json(ApiResponse::success(profile)))
}
