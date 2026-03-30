use mongodb::bson::{doc, oid::ObjectId};
use mongodb::Database;
use shared::errors::AppError;
use uuid::Uuid;

use crate::models::Notification;

pub async fn find_by_user_id(
    db: &Database,
    user_id: Uuid,
) -> Result<Vec<Notification>, AppError> {
    use futures_util::StreamExt;

    let collection = db.collection::<Notification>("notifications");
    let mut cursor = collection
        .find(doc! { "user_id": user_id.to_string() })
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let mut notifications = Vec::new();
    while let Some(result) = cursor.next().await {
        match result {
            Ok(n) => notifications.push(n),
            Err(e) => tracing::warn!("Error reading notification: {}", e),
        }
    }
    Ok(notifications)
}

pub async fn find_by_id(
    db: &Database,
    id: &str,
) -> Result<Option<Notification>, AppError> {
    let oid = ObjectId::parse_str(id)
        .map_err(|_| AppError::BadRequest("Invalid notification ID".into()))?;

    let collection = db.collection::<Notification>("notifications");
    collection
        .find_one(doc! { "_id": oid })
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
}

pub async fn insert(db: &Database, notification: &Notification) -> Result<(), AppError> {
    let collection = db.collection::<Notification>("notifications");
    collection
        .insert_one(notification)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(())
}
