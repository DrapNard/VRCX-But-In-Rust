use super::PlayerModerationType;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerModerationCreate {
    pub target_user_id: String,
    pub moderation_type: PlayerModerationType,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalAvatarModerationCreate {
    pub avatar_id: String,
    pub reason: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModerationReportCreate {
    pub target_id: String,
    pub target_type: String,
    pub subject: String,
    pub description: String,
    pub evidence: Vec<String>,
}
