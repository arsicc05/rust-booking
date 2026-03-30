use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ServiceUrls {
    pub auth: String,
    pub user: String,
    pub appointment: String,
    pub notification: String,
}

impl ServiceUrls {
    pub fn from_env() -> Self {
        Self {
            auth: std::env::var("AUTH_SERVICE_URL")
                .unwrap_or_else(|_| "http://auth-service:3001".to_string()),
            user: std::env::var("USER_SERVICE_URL")
                .unwrap_or_else(|_| "http://user-service:3002".to_string()),
            appointment: std::env::var("APPOINTMENT_SERVICE_URL")
                .unwrap_or_else(|_| "http://appointment-service:3003".to_string()),
            notification: std::env::var("NOTIFICATION_SERVICE_URL")
                .unwrap_or_else(|_| "http://notification-service:3004".to_string()),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ValidateResponse {
    pub success: bool,
    pub data: Option<ValidateData>,
}

#[derive(Debug, Deserialize)]
pub struct ValidateData {
    pub valid: bool,
    pub user_id: Option<Uuid>,
    pub email: Option<String>,
    pub role: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ComposedAppointment {
    pub appointment: serde_json::Value,
    pub notifications: Vec<serde_json::Value>,
}
