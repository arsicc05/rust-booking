use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateProfileRequest {
    pub user_id: Uuid,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
    pub bio: Option<String>,
    pub service_type: Option<String>,
    pub role: Option<String>,
    pub location_lat: Option<f64>,
    pub location_lng: Option<f64>,
    pub address: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
    pub bio: Option<String>,
    pub service_type: Option<String>,
    pub location_lat: Option<f64>,
    pub location_lng: Option<f64>,
    pub address: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ProviderQuery {
    pub service_type: Option<String>,
    pub search: Option<String>,
}
