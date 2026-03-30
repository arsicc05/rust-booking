use chrono::Utc;
use mongodb::Database;
use shared::errors::AppError;

use crate::dto::{NotificationResponse, NotificationsQuery};
use crate::models::{AppointmentEvent, Notification};
use crate::repository;

pub async fn get_notifications(
    db: &Database,
    query: NotificationsQuery,
) -> Result<Vec<NotificationResponse>, AppError> {
    let notifications = repository::find_by_user_id(db, query.user_id).await?;
    Ok(notifications.into_iter().map(NotificationResponse::from).collect())
}

pub async fn get_qr_code(
    db: &Database,
    notification_id: &str,
) -> Result<String, AppError> {
    let notification = repository::find_by_id(db, notification_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Notification not found".into()))?;

    notification
        .qr_code
        .ok_or_else(|| AppError::NotFound("No QR code for this notification".into()))
}

pub async fn handle_appointment_created(
    db: &Database,
    event: &AppointmentEvent,
) -> Result<(), AppError> {
    let qr_data = format!(
        "BOOKING:{}:{}:{}",
        event.appointment_id, event.customer_id, event.start_time
    );
    let qr_code = generate_qr_code(&qr_data)?;

    let notification = Notification {
        id: None,
        user_id: event.customer_id,
        appointment_id: event.appointment_id,
        notification_type: "booking_confirmed".to_string(),
        message: format!(
            "Your appointment has been confirmed for {} to {}",
            event.start_time, event.end_time
        ),
        qr_code: Some(qr_code),
        created_at: Utc::now(),
    };

    repository::insert(db, &notification).await?;
    tracing::info!("Created booking confirmation for appointment {}", event.appointment_id);
    Ok(())
}

pub async fn handle_appointment_cancelled(
    db: &Database,
    event: &AppointmentEvent,
) -> Result<(), AppError> {
    let notification = Notification {
        id: None,
        user_id: event.customer_id,
        appointment_id: event.appointment_id,
        notification_type: "booking_cancelled".to_string(),
        message: format!(
            "Your appointment for {} to {} has been cancelled",
            event.start_time, event.end_time
        ),
        qr_code: None,
        created_at: Utc::now(),
    };

    repository::insert(db, &notification).await?;
    tracing::info!("Created cancellation notification for appointment {}", event.appointment_id);
    Ok(())
}

fn generate_qr_code(data: &str) -> Result<String, AppError> {
    use base64::Engine;
    use image::Luma;
    use qrcode::QrCode;

    let code = QrCode::new(data.as_bytes())
        .map_err(|e| AppError::Internal(format!("QR generation failed: {}", e)))?;

    let image = code.render::<Luma<u8>>().quiet_zone(true).build();

    let mut png_bytes = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut png_bytes);
    image::ImageEncoder::write_image(
        encoder,
        image.as_raw(),
        image.width(),
        image.height(),
        image::ExtendedColorType::L8,
    )
    .map_err(|e| AppError::Internal(format!("PNG encoding failed: {}", e)))?;

    Ok(base64::engine::general_purpose::STANDARD.encode(&png_bytes))
}
