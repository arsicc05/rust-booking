use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateSlotsRequest {
    pub provider_id: Uuid,
    pub slots: Vec<SlotInput>,
}

#[derive(Debug, Deserialize)]
pub struct SlotInput {
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct BookRequest {
    pub slot_id: Uuid,
    pub customer_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct SlotsQuery {
    pub provider_id: Uuid,
    pub date: Option<String>,
    pub all: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct MyAppointmentsQuery {
    pub user_id: Option<Uuid>,
    pub role: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppointmentEvent {
    pub appointment_id: Uuid,
    pub slot_id: Uuid,
    pub customer_id: Uuid,
    pub provider_id: Uuid,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub status: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AppointmentWithDetails {
    pub id: Uuid,
    pub slot_id: Uuid,
    pub customer_id: Uuid,
    pub provider_id: Uuid,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub provider_first_name: Option<String>,
    pub provider_last_name: Option<String>,
    pub provider_service_type: Option<String>,
    pub provider_address: Option<String>,
    pub customer_first_name: Option<String>,
    pub customer_last_name: Option<String>,
}
