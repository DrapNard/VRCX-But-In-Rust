use serde::{Deserialize, Deserializer};
use serde_json::Value;

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
    #[serde(other)]
    Unknown,
}

impl Default for NotificationType {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NotificationStatus {
    Active,
    Seen,
    Hidden,
    Deleted,
    #[serde(other)]
    Unknown,
}

impl Default for NotificationStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationSender {
    #[serde(default)]
    pub id: String,

    #[serde(default)]
    pub display_name: String,

    #[serde(default)]
    pub user_icon: Option<String>,

    #[serde(default)]
    pub profile_pic_override: Option<String>,
}

#[derive(Debug, Clone, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationDetails {
    #[serde(default)]
    pub world_id: Option<String>,

    #[serde(default)]
    pub world_name: Option<String>,

    #[serde(default)]
    pub instance_id: Option<String>,

    #[serde(default)]
    pub location: Option<String>,

    #[serde(default)]
    pub image_url: Option<String>,

    #[serde(default)]
    pub response_message: Option<String>,

    #[serde(default)]
    pub raw: Value,
}

impl<'de> Deserialize<'de> for NotificationDetails {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        let parsed = match value {
            Value::String(text) => {
                serde_json::from_str::<Value>(&text).unwrap_or(Value::String(text))
            }
            other => other,
        };

        Ok(Self {
            world_id: string_field(&parsed, "worldId"),
            world_name: string_field(&parsed, "worldName"),
            instance_id: string_field(&parsed, "instanceId"),
            location: string_field(&parsed, "location"),
            image_url: string_field(&parsed, "imageUrl"),
            response_message: string_field(&parsed, "responseMessage"),
            raw: parsed,
        })
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationV2 {
    #[serde(default)]
    pub id: String,

    #[serde(default)]
    pub receiver_user_id: String,

    #[serde(default)]
    pub sender_user_id: Option<String>,

    #[serde(default)]
    pub sender: Option<NotificationSender>,

    #[serde(default)]
    pub notification_type: NotificationType,

    #[serde(default)]
    pub message: Option<String>,

    #[serde(default)]
    pub details: NotificationDetails,

    #[serde(default)]
    pub seen: bool,

    #[serde(default)]
    pub status: NotificationStatus,

    #[serde(default)]
    pub created_at: String,

    #[serde(default)]
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

fn string_field(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(ToString::to_string)
}
