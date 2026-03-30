use mongodb::Database;

use crate::models::AppointmentEvent;
use crate::service;

pub async fn handle_appointment_created(
    db: &Database,
    event: &AppointmentEvent,
) -> anyhow::Result<()> {
    service::handle_appointment_created(db, event)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))
}

pub async fn handle_appointment_cancelled(
    db: &Database,
    event: &AppointmentEvent,
) -> anyhow::Result<()> {
    service::handle_appointment_cancelled(db, event)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))
}
