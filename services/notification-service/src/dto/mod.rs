use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct NotificationsQuery {
    pub user_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct NotificationResponse {
    pub id: String,
    pub user_id: String,
    pub appointment_id: String,
    pub notification_type: String,
    pub message: String,
    pub qr_code: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct ValidateQrRequest {
    pub qr_data: String,
}

#[derive(Debug, Serialize)]
pub struct ValidateQrResponse {
    pub valid: bool,
    pub appointment_id: Option<String>,
    pub customer_id: Option<String>,
    pub start_time: Option<String>,
}

impl From<crate::models::Notification> for NotificationResponse {
    fn from(n: crate::models::Notification) -> Self {
        Self {
            id: n.id.map(|oid| oid.to_hex()).unwrap_or_default(),
            user_id: n.user_id,
            appointment_id: n.appointment_id,
            notification_type: n.notification_type,
            message: n.message,
            qr_code: n.qr_code,
            created_at: n.created_at,
        }
    }
}
