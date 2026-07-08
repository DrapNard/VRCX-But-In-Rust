use crate::models::users::UserSummary;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FriendStatus {
    Friend,
    NotFriend,
    IncomingRequest,
    OutgoingRequest,
    Blocked,
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

pub type FriendList = crate::models::common::Paginated<FriendSummary>;

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
