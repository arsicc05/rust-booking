use shared::errors::AppError;
use sqlx::PgPool;
use uuid::Uuid;

use crate::dto::{CreateProfileRequest, UpdateProfileRequest};
use crate::models::Profile;

pub async fn find_by_user_id(pool: &PgPool, user_id: Uuid) -> Result<Option<Profile>, AppError> {
    sqlx::query_as::<_, Profile>("SELECT * FROM profiles WHERE user_id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
}

pub async fn create(pool: &PgPool, req: &CreateProfileRequest) -> Result<Profile, AppError> {
    sqlx::query_as::<_, Profile>(
        r#"INSERT INTO profiles (user_id, first_name, last_name, phone, bio, service_type, location_lat, location_lng, address)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING *"#,
    )
    .bind(req.user_id)
    .bind(&req.first_name)
    .bind(&req.last_name)
    .bind(&req.phone)
    .bind(&req.bio)
    .bind(&req.service_type)
    .bind(req.location_lat)
    .bind(req.location_lng)
    .bind(&req.address)
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))
}

pub async fn update(
    pool: &PgPool,
    user_id: Uuid,
    req: &UpdateProfileRequest,
) -> Result<Option<Profile>, AppError> {
    sqlx::query_as::<_, Profile>(
        r#"UPDATE profiles SET
           first_name = COALESCE($2, first_name),
           last_name = COALESCE($3, last_name),
           phone = COALESCE($4, phone),
           bio = COALESCE($5, bio),
           service_type = COALESCE($6, service_type),
           location_lat = COALESCE($7, location_lat),
           location_lng = COALESCE($8, location_lng),
           address = COALESCE($9, address),
           updated_at = NOW()
           WHERE user_id = $1 RETURNING *"#,
    )
    .bind(user_id)
    .bind(&req.first_name)
    .bind(&req.last_name)
    .bind(&req.phone)
    .bind(&req.bio)
    .bind(&req.service_type)
    .bind(req.location_lat)
    .bind(req.location_lng)
    .bind(&req.address)
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))
}

pub async fn find_providers(
    pool: &PgPool,
    service_type: Option<&str>,
    search: Option<&str>,
) -> Result<Vec<Profile>, AppError> {
    if let Some(st) = service_type {
        sqlx::query_as::<_, Profile>(
            "SELECT * FROM profiles WHERE service_type = $1 ORDER BY created_at DESC",
        )
        .bind(st)
        .fetch_all(pool)
        .await
    } else if let Some(s) = search {
        let pattern = format!("%{}%", s);
        sqlx::query_as::<_, Profile>(
            r#"SELECT * FROM profiles WHERE
               first_name ILIKE $1 OR last_name ILIKE $1 OR service_type ILIKE $1
               ORDER BY created_at DESC"#,
        )
        .bind(&pattern)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as::<_, Profile>(
            "SELECT * FROM profiles WHERE service_type IS NOT NULL ORDER BY created_at DESC",
        )
        .fetch_all(pool)
        .await
    }
    .map_err(|e| AppError::Internal(e.to_string()))
}

pub async fn find_provider(pool: &PgPool, user_id: Uuid) -> Result<Option<Profile>, AppError> {
    sqlx::query_as::<_, Profile>(
        "SELECT * FROM profiles WHERE user_id = $1 AND service_type IS NOT NULL",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))
}

pub async fn update_avatar(
    pool: &PgPool,
    user_id: Uuid,
    avatar_url: &str,
) -> Result<Option<Profile>, AppError> {
    sqlx::query_as::<_, Profile>(
        "UPDATE profiles SET avatar_url = $2, updated_at = NOW() WHERE user_id = $1 RETURNING *",
    )
    .bind(user_id)
    .bind(avatar_url)
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))
}
