#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PlayerModerationType {
    Block,
    Mute,
    HideAvatar,
    InteractOff,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerModeration {
    pub id: String,
    pub source_user_id: String,
    pub target_user_id: String,
    #[serde(rename = "type")]
    pub moderation_type: PlayerModerationType,
    #[serde(rename = "created", alias = "created_at")]
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

#[cfg(test)]
mod tests {
    use super::{PlayerModeration, PlayerModerationType};

    #[test]
    fn decodes_current_player_moderation_fields() {
        let moderation: PlayerModeration = serde_json::from_str(
            r#"{"id":"pmod_1","sourceUserId":"usr_1","targetUserId":"usr_2","type":"block","created":"2026-01-01T00:00:00Z"}"#,
        )
        .unwrap();

        assert!(matches!(
            moderation.moderation_type,
            PlayerModerationType::Block
        ));
        assert_eq!(moderation.created_at, "2026-01-01T00:00:00Z");
    }
}
