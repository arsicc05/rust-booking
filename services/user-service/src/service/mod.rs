use shared::errors::AppError;
use sqlx::PgPool;
use uuid::Uuid;

use crate::dto::{CreateProfileRequest, UpdateProfileRequest};
use crate::models::Profile;
use crate::repository;

pub async fn get_profile(pool: &PgPool, user_id: Uuid) -> Result<Profile, AppError> {
    repository::find_by_user_id(pool, user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Profile not found".into()))
}

pub async fn create_profile(pool: &PgPool, req: CreateProfileRequest) -> Result<Profile, AppError> {
    repository::create(pool, &req).await
}

pub async fn update_profile(
    pool: &PgPool,
    user_id: Uuid,
    req: UpdateProfileRequest,
) -> Result<Profile, AppError> {
    repository::update(pool, user_id, &req)
        .await?
        .ok_or_else(|| AppError::NotFound("Profile not found".into()))
}

pub async fn list_providers(
    pool: &PgPool,
    service_type: Option<&str>,
    search: Option<&str>,
) -> Result<Vec<Profile>, AppError> {
    repository::find_providers(pool, service_type, search).await
}

pub async fn get_provider(pool: &PgPool, user_id: Uuid) -> Result<Profile, AppError> {
    repository::find_provider(pool, user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Provider not found".into()))
}

pub async fn upload_avatar(
    pool: &PgPool,
    user_id: Uuid,
    file_name: &str,
    file_data: &[u8],
) -> Result<Profile, AppError> {
    if file_data.len() > 5 * 1024 * 1024 {
        return Err(AppError::BadRequest("File too large (max 5MB)".into()));
    }

    let upload_dir = std::path::Path::new("./uploads");
    tokio::fs::create_dir_all(upload_dir)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let ext = std::path::Path::new(file_name)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png");
    let stored_name = format!("{}_{}.{}", user_id, chrono::Utc::now().timestamp(), ext);
    let file_path = upload_dir.join(&stored_name);

    tokio::fs::write(&file_path, file_data)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let avatar_url = format!("/uploads/{}", stored_name);

    repository::update_avatar(pool, user_id, &avatar_url)
        .await?
        .ok_or_else(|| AppError::NotFound("Profile not found".into()))
}
