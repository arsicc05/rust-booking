use shared::errors::AppError;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::User;

pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<Option<User>, AppError> {
    sqlx::query_as::<_, User>("SELECT id, email, password_hash, role::TEXT, created_at, updated_at FROM users WHERE email = $1")
        .bind(email)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<User>, AppError> {
    sqlx::query_as::<_, User>("SELECT id, email, password_hash, role::TEXT, created_at, updated_at FROM users WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
}

pub async fn email_exists(pool: &PgPool, email: &str) -> Result<bool, AppError> {
    let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE email = $1")
        .bind(email)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(count > 0)
}

pub async fn create(
    pool: &PgPool,
    email: &str,
    password_hash: &str,
    role: &str,
) -> Result<User, AppError> {
    sqlx::query_as::<_, User>(
        "INSERT INTO users (email, password_hash, role) VALUES ($1, $2, $3::user_role) RETURNING id, email, password_hash, role::TEXT, created_at, updated_at",
    )
    .bind(email)
    .bind(password_hash)
    .bind(role)
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))
}
