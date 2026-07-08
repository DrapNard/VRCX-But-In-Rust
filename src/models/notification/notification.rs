#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NotificationType {
    FriendRequest,
    Invite,
    InviteResponse,
    RequestInvite,
    RequestInviteResponse,
    VoteToKick,
    Group,
    GroupAnnouncement,
    GroupInvite,
    GroupJoinRequest,
    Unknown,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NotificationStatus {
    Active,
    Seen,
    Hidden,
    Deleted,
    Unknown,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationSender {
    pub id: String,
    pub display_name: String,
    pub user_icon: Option<String>,
    pub profile_pic_override: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationDetails {
    pub world_id: Option<String>,
    pub world_name: Option<String>,
    pub instance_id: Option<String>,
    pub location: Option<String>,
    pub image_url: Option<String>,
    pub response_message: Option<String>,
    pub raw: serde_json::Value,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationV2 {
    pub id: String,
    pub receiver_user_id: String,
    pub sender_user_id: Option<String>,
    pub sender: Option<NotificationSender>,
    pub notification_type: NotificationType,
    pub message: Option<String>,
    pub details: NotificationDetails,
    pub seen: bool,
    pub status: NotificationStatus,
    pub created_at: String,
    pub expires_at: Option<String>,
}

pub type NotificationList = crate::models::common::Paginated<NotificationV2>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationResponse {
    pub notification: Option<NotificationV2>,
    pub message: Option<String>,
    pub status_code: Option<u16>,
}
