use shared::errors::AppError;
use sqlx::PgPool;
use uuid::Uuid;

use crate::dto::{
    AppointmentEvent, BookRequest, CreateSlotsRequest, MyAppointmentsQuery, SlotsQuery,
};
use crate::models::{Appointment, TimeSlot};
use crate::repository;

pub async fn create_slots(
    pool: &PgPool,
    payload: CreateSlotsRequest,
) -> Result<Vec<TimeSlot>, AppError> {
    if payload.slots.is_empty() {
        return Err(AppError::BadRequest("At least one slot is required".into()));
    }

    let slot_tuples: Vec<_> = payload
        .slots
        .iter()
        .map(|s| (s.start_time, s.end_time))
        .collect();

    repository::create_slots(pool, payload.provider_id, &slot_tuples).await
}

pub async fn get_available_slots(
    pool: &PgPool,
    query: SlotsQuery,
) -> Result<Vec<TimeSlot>, AppError> {
    repository::find_available_slots(pool, query.provider_id, query.date.as_deref()).await
}

pub async fn book_appointment(
    pool: &PgPool,
    nats: &Option<async_nats::Client>,
    payload: BookRequest,
) -> Result<Appointment, AppError> {
    // Saga pattern: begin tx → lock slot → update status → create appointment → commit
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let slot = repository::lock_slot_for_update(&mut tx, payload.slot_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Time slot not found".into()))?;

    if slot.status != "available" {
        return Err(AppError::Conflict(
            "Time slot is no longer available".into(),
        ));
    }

    repository::update_slot_status(&mut tx, payload.slot_id, "booked").await?;

    let appointment = repository::create_appointment(
        &mut tx,
        payload.slot_id,
        payload.customer_id,
        slot.provider_id,
    )
    .await?;

    tx.commit()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    publish_event(
        nats,
        "appointments.created",
        &AppointmentEvent {
            appointment_id: appointment.id,
            slot_id: appointment.slot_id,
            customer_id: appointment.customer_id,
            provider_id: appointment.provider_id,
            start_time: slot.start_time,
            end_time: slot.end_time,
            status: "confirmed".to_string(),
        },
    )
    .await;

    Ok(appointment)
}

pub async fn cancel_appointment(
    pool: &PgPool,
    nats: &Option<async_nats::Client>,
    appointment_id: Uuid,
) -> Result<(), AppError> {
    let appointment = repository::find_appointment_by_id(pool, appointment_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Appointment not found".into()))?;

    if appointment.status == "cancelled" {
        return Err(AppError::Conflict("Appointment already cancelled".into()));
    }

    repository::cancel_appointment(pool, appointment_id).await?;
    repository::release_slot(pool, appointment.slot_id).await?;

    let slot = repository::find_slot_by_id(pool, appointment.slot_id).await?;
    let (start_time, end_time) = slot.map(|s| (s.start_time, s.end_time)).unwrap_or_default();

    publish_event(
        nats,
        "appointments.cancelled",
        &AppointmentEvent {
            appointment_id,
            slot_id: appointment.slot_id,
            customer_id: appointment.customer_id,
            provider_id: appointment.provider_id,
            start_time,
            end_time,
            status: "cancelled".to_string(),
        },
    )
    .await;

    Ok(())
}

pub async fn my_appointments(
    pool: &PgPool,
    query: MyAppointmentsQuery,
) -> Result<Vec<Appointment>, AppError> {
    match query.role.as_deref() {
        Some("provider") => repository::find_appointments_by_provider(pool, query.user_id).await,
        _ => repository::find_appointments_by_customer(pool, query.user_id).await,
    }
}

async fn publish_event(nats: &Option<async_nats::Client>, subject: &str, event: &AppointmentEvent) {
    if let Some(client) = nats {
        match serde_json::to_vec(event) {
            Ok(payload) => {
                if let Err(e) = client.publish(subject.to_string(), payload.into()).await {
                    tracing::error!("Failed to publish {}: {}", subject, e);
                } else {
                    tracing::info!(
                        "Published {} for appointment {}",
                        subject,
                        event.appointment_id
                    );
                }
            }
            Err(e) => tracing::error!("Failed to serialize event: {}", e),
        }
    }
}
