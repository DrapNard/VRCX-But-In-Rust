use super::{NotificationDetails, NotificationSender, NotificationType};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Notification {
    pub id: String,
    pub sender_user_id: Option<String>,
    pub sender: Option<NotificationSender>,
    pub receiver_user_id: String,
    pub notification_type: NotificationType,
    pub message: Option<String>,
    pub details: NotificationDetails,
    pub seen: bool,
    pub created_at: String,
}

pub type NotificationLegacyList = crate::models::common::Paginated<Notification>;
