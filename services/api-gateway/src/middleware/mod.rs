use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::header;
use axum::middleware::Next;
use axum::response::Response;
use shared::auth;
use shared::errors::AppError;

use crate::AppState;

pub async fn auth_middleware(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    let path = req.uri().path().to_string();

    if path.contains("/auth/login")
        || path.contains("/auth/register")
        || path.starts_with("/uploads/")
        || path == "/health"
    {
        return Ok(next.run(req).await);
    }

    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("Missing Authorization header".into()))?;

    let token = auth::extract_token_from_header(auth_header)
        .ok_or_else(|| AppError::Unauthorized("Invalid authorization format".into()))?;

    let claims = auth::validate_token(token, &state.jwt_config.secret)
        .map_err(|_| AppError::Unauthorized("Invalid or expired token".into()))?;

    let mut req = req;
    req.headers_mut()
        .insert("X-User-Id", claims.sub.to_string().parse().unwrap());
    req.headers_mut().insert(
        "X-User-Role",
        format!("{:?}", claims.role).to_lowercase().parse().unwrap(),
    );

    Ok(next.run(req).await)
}
