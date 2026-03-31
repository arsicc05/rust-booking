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
    pub has_qr_code: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<crate::models::Notification> for NotificationResponse {
    fn from(n: crate::models::Notification) -> Self {
        Self {
            id: n.id.map(|oid| oid.to_hex()).unwrap_or_default(),
            user_id: n.user_id,
            appointment_id: n.appointment_id,
            notification_type: n.notification_type,
            message: n.message,
            has_qr_code: n.qr_code.is_some(),
            created_at: n.created_at,
        }
    }
}
