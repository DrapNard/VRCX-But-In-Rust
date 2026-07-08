#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PlayerModerationType {
    Block,
    Mute,
    HideAvatar,
    InteractOff,
    Unknown,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerModeration {
    pub id: String,
    pub source_user_id: String,
    pub target_user_id: String,
    pub moderation_type: PlayerModerationType,
    pub created_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalAvatarModeration {
    pub id: String,
    pub avatar_id: String,
    pub reason: String,
    pub created_at: String,
}
