use crate::models::users::UserSummary;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FriendStatus {
    Friend,
    NotFriend,
    IncomingRequest,
    OutgoingRequest,
    Blocked,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FriendSummary {
    pub user: UserSummary,
    pub status: FriendStatus,
    pub note: Option<String>,
    pub friend_key: Option<String>,
}

pub type FriendList = Vec<Friend>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Friend {
    #[serde(default)]
    pub id: String,

    #[serde(default)]
    pub display_name: String,

    #[serde(default)]
    pub bio: Option<String>,

    #[serde(default)]
    pub bio_links: Vec<String>,

    #[serde(default)]
    pub current_avatar_image_url: Option<String>,

    #[serde(default)]
    pub current_avatar_tags: Vec<String>,

    #[serde(default)]
    pub current_avatar_thumbnail_image_url: Option<String>,

    #[serde(default)]
    pub developer_type: Option<String>,

    #[serde(default)]
    pub friend_key: Option<String>,

    #[serde(default)]
    pub image_url: Option<String>,

    #[serde(default)]
    pub is_friend: Option<bool>,

    #[serde(default)]
    pub last_activity: Option<String>,

    #[serde(default)]
    pub last_login: Option<String>,

    #[serde(default)]
    pub last_mobile: Option<String>,

    #[serde(default)]
    pub last_platform: Option<String>,

    #[serde(default)]
    pub location: Option<String>,

    #[serde(default)]
    pub platform: Option<String>,

    #[serde(default)]
    pub profile_pic_override: Option<String>,

    #[serde(default)]
    pub profile_pic_override_thumbnail: Option<String>,

    #[serde(default)]
    pub status: Option<String>,

    #[serde(default)]
    pub status_description: Option<String>,

    #[serde(default)]
    pub tags: Vec<String>,

    #[serde(default)]
    pub user_icon: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FriendRequest {
    pub id: String,
    pub sender_id: String,
    pub receiver_id: String,
    pub status: FriendStatus,
    pub message: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Boop {
    pub id: String,
    pub sender_id: String,
    pub receiver_id: String,
    pub message: Option<String>,
    pub created_at: String,
}
