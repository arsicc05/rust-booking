use axum::extract::{Path, Query, State};
use axum::Json;
use shared::errors::AppError;
use shared::models::ApiResponse;
use uuid::Uuid;

use crate::dto::*;
use crate::models::{Appointment, TimeSlot};
use crate::service;
use crate::AppState;

pub async fn create_slots(
    State(state): State<AppState>,
    Json(payload): Json<CreateSlotsRequest>,
) -> Result<Json<ApiResponse<Vec<TimeSlot>>>, AppError> {
    let slots = service::create_slots(&state.db, payload).await?;
    Ok(Json(ApiResponse::success(slots)))
}

pub async fn get_available_slots(
    State(state): State<AppState>,
    Query(query): Query<SlotsQuery>,
) -> Result<Json<ApiResponse<Vec<TimeSlot>>>, AppError> {
    let slots = service::get_available_slots(&state.db, query).await?;
    Ok(Json(ApiResponse::success(slots)))
}

pub async fn book_appointment(
    State(state): State<AppState>,
    Json(payload): Json<BookRequest>,
) -> Result<Json<ApiResponse<Appointment>>, AppError> {
    let appointment = service::book_appointment(&state.db, &state.nats, payload).await?;
    Ok(Json(ApiResponse::success(appointment)))
}

pub async fn cancel_appointment(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    service::cancel_appointment(&state.db, &state.nats, id).await?;
    Ok(Json(ApiResponse::success("Appointment cancelled".to_string())))
}

pub async fn my_appointments(
    State(state): State<AppState>,
    Query(query): Query<MyAppointmentsQuery>,
) -> Result<Json<ApiResponse<Vec<Appointment>>>, AppError> {
    let appointments = service::my_appointments(&state.db, query).await?;
    Ok(Json(ApiResponse::success(appointments)))
}
