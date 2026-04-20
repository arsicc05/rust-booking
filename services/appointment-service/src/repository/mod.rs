use shared::errors::AppError;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{Appointment, TimeSlot};

pub async fn create_slots(
    pool: &PgPool,
    provider_id: Uuid,
    slots: &[(chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)],
) -> Result<Vec<TimeSlot>, AppError> {
    let mut created = Vec::with_capacity(slots.len());
    for (start, end) in slots {
        let slot = sqlx::query_as::<_, TimeSlot>(
            "INSERT INTO time_slots (provider_id, start_time, end_time) VALUES ($1, $2, $3) RETURNING id, provider_id, start_time, end_time, status::TEXT as status, created_at",
        )
        .bind(provider_id)
        .bind(start)
        .bind(end)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
        created.push(slot);
    }
    Ok(created)
}

pub async fn find_available_slots(
    pool: &PgPool,
    provider_id: Uuid,
    date: Option<&str>,
) -> Result<Vec<TimeSlot>, AppError> {
    let slots = if let Some(date_str) = date {
        sqlx::query_as::<_, TimeSlot>(
            "SELECT id, provider_id, start_time, end_time, status::TEXT as status, created_at FROM time_slots WHERE provider_id = $1 AND status = 'available' AND start_time::date = $2::date ORDER BY start_time",
        )
        .bind(provider_id)
        .bind(date_str)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as::<_, TimeSlot>(
            "SELECT id, provider_id, start_time, end_time, status::TEXT as status, created_at FROM time_slots WHERE provider_id = $1 AND status = 'available' ORDER BY start_time",
        )
        .bind(provider_id)
        .fetch_all(pool)
        .await
    };
    slots.map_err(|e| AppError::Internal(e.to_string()))
}

pub async fn find_all_provider_slots(
    pool: &PgPool,
    provider_id: Uuid,
) -> Result<Vec<TimeSlot>, AppError> {
    sqlx::query_as::<_, TimeSlot>(
        "SELECT id, provider_id, start_time, end_time, status::TEXT as status, created_at FROM time_slots WHERE provider_id = $1 ORDER BY start_time",
    )
    .bind(provider_id)
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))
}

pub async fn lock_slot_for_update(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    slot_id: Uuid,
) -> Result<Option<TimeSlot>, AppError> {
    sqlx::query_as::<_, TimeSlot>(
        "SELECT id, provider_id, start_time, end_time, status::TEXT as status, created_at FROM time_slots WHERE id = $1 FOR UPDATE",
    )
    .bind(slot_id)
    .fetch_optional(tx.as_mut())
    .await
    .map_err(|e| AppError::Internal(e.to_string()))
}

pub async fn update_slot_status(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    slot_id: Uuid,
    status: &str,
) -> Result<(), AppError> {
    sqlx::query("UPDATE time_slots SET status = $1::slot_status WHERE id = $2")
        .bind(status)
        .bind(slot_id)
        .execute(tx.as_mut())
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(())
}

pub async fn create_appointment(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    slot_id: Uuid,
    customer_id: Uuid,
    provider_id: Uuid,
) -> Result<Appointment, AppError> {
    sqlx::query_as::<_, Appointment>(
        "INSERT INTO appointments (slot_id, customer_id, provider_id, status) VALUES ($1, $2, $3, 'confirmed') RETURNING id, slot_id, customer_id, provider_id, status::TEXT as status, created_at, updated_at",
    )
    .bind(slot_id)
    .bind(customer_id)
    .bind(provider_id)
    .fetch_one(tx.as_mut())
    .await
    .map_err(|e| AppError::Internal(e.to_string()))
}

pub async fn find_appointment_by_id(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<Appointment>, AppError> {
    sqlx::query_as::<_, Appointment>("SELECT id, slot_id, customer_id, provider_id, status::TEXT as status, created_at, updated_at FROM appointments WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
}

pub async fn cancel_appointment(pool: &PgPool, appointment_id: Uuid) -> Result<(), AppError> {
    sqlx::query("UPDATE appointments SET status = 'cancelled', updated_at = NOW() WHERE id = $1")
        .bind(appointment_id)
        .execute(pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(())
}

pub async fn release_slot(pool: &PgPool, slot_id: Uuid) -> Result<(), AppError> {
    sqlx::query("UPDATE time_slots SET status = 'available'::slot_status WHERE id = $1")
        .bind(slot_id)
        .execute(pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(())
}

pub async fn find_slot_by_id(pool: &PgPool, slot_id: Uuid) -> Result<Option<TimeSlot>, AppError> {
    sqlx::query_as::<_, TimeSlot>("SELECT id, provider_id, start_time, end_time, status::TEXT as status, created_at FROM time_slots WHERE id = $1")
        .bind(slot_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
}

pub async fn find_appointments_by_customer(
    pool: &PgPool,
    customer_id: Uuid,
) -> Result<Vec<Appointment>, AppError> {
    sqlx::query_as::<_, Appointment>(
        "SELECT 
            a.id, a.slot_id, a.customer_id, a.provider_id, 
            a.status::TEXT as status, a.created_at, a.updated_at,
            ts.start_time, ts.end_time
         FROM appointments a
         JOIN time_slots ts ON a.slot_id = ts.id
         WHERE a.customer_id = $1 
         ORDER BY ts.start_time DESC",
    )
    .bind(customer_id)
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))
}

pub async fn find_appointments_by_provider(
    pool: &PgPool,
    provider_id: Uuid,
) -> Result<Vec<Appointment>, AppError> {
    sqlx::query_as::<_, Appointment>(
        "SELECT 
            a.id, a.slot_id, a.customer_id, a.provider_id, 
            a.status::TEXT as status, a.created_at, a.updated_at,
            ts.start_time, ts.end_time
         FROM appointments a
         JOIN time_slots ts ON a.slot_id = ts.id
         WHERE a.provider_id = $1 
         ORDER BY ts.start_time DESC",
    )
    .bind(provider_id)
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))
}
