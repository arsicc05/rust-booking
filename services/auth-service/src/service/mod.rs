use shared::auth;
use shared::config::JwtConfig;
use shared::errors::AppError;
use shared::models::UserRole;
use sqlx::PgPool;

use crate::dto::{AuthResponse, RegisterRequest, ValidateResponse};
use crate::repository;

pub async fn register(
    pool: &PgPool,
    jwt_config: &JwtConfig,
    payload: RegisterRequest,
) -> Result<AuthResponse, AppError> {
    if payload.email.is_empty() || payload.password.is_empty() {
        return Err(AppError::BadRequest("Email and password are required".into()));
    }
    if payload.password.len() < 6 {
        return Err(AppError::BadRequest(
            "Password must be at least 6 characters".into(),
        ));
    }

    let role = UserRole::from_str(&payload.role)
        .ok_or_else(|| AppError::BadRequest("Invalid role. Must be: user, provider, admin".into()))?;

    if repository::email_exists(pool, &payload.email).await? {
        return Err(AppError::Conflict("User with this email already exists".into()));
    }

    let password_hash = hash_password(&payload.password)?;
    let user = repository::create(pool, &payload.email, &password_hash, &payload.role).await?;

    let access_token = generate_access_token(user.id, &user.email, &role, jwt_config)?;
    let refresh_token = generate_refresh_token(user.id, &user.email, &role, jwt_config)?;

    Ok(AuthResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        user_id: user.id,
        email: user.email,
        role: user.role,
    })
}

pub async fn login(
    pool: &PgPool,
    jwt_config: &JwtConfig,
    email: &str,
    password: &str,
) -> Result<AuthResponse, AppError> {
    let user = repository::find_by_email(pool, email)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Invalid email or password".into()))?;

    if !verify_password(password, &user.password_hash)? {
        return Err(AppError::Unauthorized("Invalid email or password".into()));
    }

    let role = UserRole::from_str(&user.role)
        .ok_or_else(|| AppError::Internal("Invalid role in database".into()))?;

    let access_token = generate_access_token(user.id, &user.email, &role, jwt_config)?;
    let refresh_token = generate_refresh_token(user.id, &user.email, &role, jwt_config)?;

    Ok(AuthResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        user_id: user.id,
        email: user.email,
        role: user.role,
    })
}

pub async fn refresh(
    pool: &PgPool,
    jwt_config: &JwtConfig,
    refresh_token_str: &str,
) -> Result<AuthResponse, AppError> {
    let claims = auth::validate_token(refresh_token_str, &jwt_config.secret)
        .map_err(|_| AppError::Unauthorized("Invalid or expired refresh token".into()))?;

    let user = repository::find_by_id(pool, claims.sub)
        .await?
        .ok_or_else(|| AppError::Unauthorized("User no longer exists".into()))?;

    let role = UserRole::from_str(&user.role)
        .ok_or_else(|| AppError::Internal("Invalid role in database".into()))?;

    let access_token = generate_access_token(user.id, &user.email, &role, jwt_config)?;
    let new_refresh = generate_refresh_token(user.id, &user.email, &role, jwt_config)?;

    Ok(AuthResponse {
        access_token,
        refresh_token: new_refresh,
        token_type: "Bearer".to_string(),
        user_id: user.id,
        email: user.email,
        role: user.role,
    })
}

pub fn validate_token(jwt_config: &JwtConfig, token: &str) -> ValidateResponse {
    match auth::validate_token(token, &jwt_config.secret) {
        Ok(claims) => ValidateResponse {
            valid: true,
            user_id: Some(claims.sub),
            email: Some(claims.email),
            role: Some(claims.role.to_string()),
        },
        Err(_) => ValidateResponse {
            valid: false,
            user_id: None,
            email: None,
            role: None,
        },
    }
}

// --- private helpers ---

fn generate_access_token(
    user_id: uuid::Uuid,
    email: &str,
    role: &UserRole,
    config: &JwtConfig,
) -> Result<String, AppError> {
    auth::create_token(user_id, email, role, &config.secret, config.expiry_hours)
        .map_err(|e| AppError::Internal(e.to_string()))
}

fn generate_refresh_token(
    user_id: uuid::Uuid,
    email: &str,
    role: &UserRole,
    config: &JwtConfig,
) -> Result<String, AppError> {
    auth::create_token(user_id, email, role, &config.secret, config.refresh_expiry_hours)
        .map_err(|e| AppError::Internal(e.to_string()))
}

fn hash_password(password: &str) -> Result<String, AppError> {
    use argon2::{
        password_hash::{rand_core::OsRng, SaltString},
        Argon2, PasswordHasher,
    };
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| AppError::Internal(format!("Failed to hash password: {}", e)))
}

fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    use argon2::{password_hash::PasswordHash, Argon2, PasswordVerifier};
    let parsed = PasswordHash::new(hash)
        .map_err(|e| AppError::Internal(format!("Failed to parse hash: {}", e)))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}
