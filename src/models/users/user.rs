use super::{Badges, Identity, Metadata, Presence, Profile, Social, Tags};
use crate::models::users::{
    AdminTags, Permissions, SupporterState, TrollState, TrustRank,
    identity::{StatusInfo, UserStatus},
};
use serde::{Deserialize, Deserializer};
use serde_json::Value;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSummary {
    pub identity: Identity,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub profile: Profile,
    pub identity: Identity,
    pub metadata: Metadata,
    pub presence: Presence,
    pub social: Social,
    pub badges: Vec<Badges>,
    pub tags: Tags,
    pub age_verified: bool,
    pub age_verification_status: String,
    pub requires_two_factor_auth: Option<Vec<String>>,
}

impl<'de> Deserialize<'de> for User {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;

        let requires_two_factor_auth =
            string_array(&value, "requiresTwoFactorAuth").filter(|methods| !methods.is_empty());

        if value.get("identity").is_some() {
            return deserialize_legacy_user(value, requires_two_factor_auth)
                .map_err(serde::de::Error::custom);
        }

        Ok(User {
            profile: Profile {
                bio: string_field(&value, "bio"),
                bio_links: string_array(&value, "bioLinks").unwrap_or_default(),
                pronouns: string_field(&value, "pronouns"),
                user_icon: string_field(&value, "userIcon"),
                profile_pic_override: string_field(&value, "profilePicOverride"),
                profile_pic_override_thumbnail: string_field(&value, "profilePicOverrideThumbnail"),
                current_avatar_image_url: string_field(&value, "currentAvatarImageUrl"),
                current_avatar_thumbnail_image_url: string_field(
                    &value,
                    "currentAvatarThumbnailImageUrl",
                ),
            },
            identity: Identity {
                id: string_field(&value, "id"),
                username: string_field(&value, "username"),
                display_name: string_field(&value, "displayName"),
                status: StatusInfo {
                    status: user_status(&string_field(&value, "status")),
                    status_description: string_field(&value, "statusDescription"),
                },
            },
            metadata: Metadata {
                date_joined: string_field(&value, "date_joined"),
                last_activity: string_field(&value, "last_activity"),
                last_login: string_field(&value, "last_login"),
                last_mobile: string_field(&value, "last_mobile"),
            },
            presence: Presence {
                instance: None,
                traveling_to_instance: None,
                travels: Vec::new(),
                platform: string_field_from_presence(&value, "platform")
                    .unwrap_or_else(|| string_field(&value, "last_platform")),
                last_platform: string_field(&value, "last_platform"),
            },
            social: Social {
                is_friend: bool_field(&value, "isFriend"),
                friend_key: string_field(&value, "friendKey"),
                friend_request_status: string_field(&value, "friendRequestStatus"),
                mutuals: Vec::new(),
                groups: Vec::new(),
                note: string_field(&value, "note"),
            },
            badges: serde_json::from_value(value.get("badges").cloned().unwrap_or(Value::Null))
                .unwrap_or_default(),
            tags: tags_from_raw(string_array(&value, "tags").unwrap_or_default()),
            age_verified: bool_field(&value, "ageVerified"),
            age_verification_status: string_field(&value, "ageVerificationStatus"),
            requires_two_factor_auth,
        })
    }
}

fn deserialize_legacy_user(
    value: Value,
    requires_two_factor_auth: Option<Vec<String>>,
) -> serde_json::Result<User> {
    let raw = serde_json::from_value::<LegacyUser>(value)?;
    let mut user = User {
        profile: raw.profile,
        identity: raw.identity,
        metadata: raw.metadata,
        presence: raw.presence,
        social: raw.social,
        badges: raw.badges,
        tags: raw.tags,
        age_verified: raw.age_verified,
        age_verification_status: raw.age_verification_status,
        requires_two_factor_auth: raw.requires_two_factor_auth,
    };
    user.requires_two_factor_auth = requires_two_factor_auth;
    Ok(user)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LegacyUser {
    profile: Profile,
    identity: Identity,
    metadata: Metadata,
    presence: Presence,
    social: Social,
    badges: Vec<Badges>,
    tags: Tags,
    #[serde(default)]
    age_verified: bool,
    #[serde(default)]
    age_verification_status: String,
    requires_two_factor_auth: Option<Vec<String>>,
}

fn string_field(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn bool_field(value: &Value, key: &str) -> bool {
    value.get(key).and_then(Value::as_bool).unwrap_or_default()
}

fn string_array(value: &Value, key: &str) -> Option<Vec<String>> {
    value.get(key).and_then(|items| {
        Some(
            items
                .as_array()?
                .iter()
                .filter_map(Value::as_str)
                .map(ToString::to_string)
                .collect(),
        )
    })
}

fn string_field_from_presence(value: &Value, key: &str) -> Option<String> {
    value
        .get("presence")
        .and_then(|presence| presence.get(key))
        .and_then(Value::as_str)
        .map(ToString::to_string)
}

fn user_status(status: &str) -> UserStatus {
    match status {
        "active" => UserStatus::Active,
        "joinMe" => UserStatus::JoinMe,
        "askMe" => UserStatus::AskMe,
        "busy" => UserStatus::Busy,
        "offline" => UserStatus::Offline,
        _ => UserStatus::Unknown,
    }
}

fn tags_from_raw(raw: Vec<String>) -> Tags {
    Tags {
        trust_rank: trust_rank(&raw),
        troll_state: troll_state(&raw),
        admin_tags: raw.iter().filter_map(|tag| admin_tag(tag)).collect(),
        supporter_state: supporter_state(&raw),
        permissions: raw.iter().filter_map(|tag| permission(tag)).collect(),
        languages: raw
            .iter()
            .filter_map(|tag| tag.strip_prefix("language_").map(ToString::to_string))
            .collect(),
        raw,
    }
}

fn trust_rank(tags: &[String]) -> TrustRank {
    if tags.iter().any(|tag| tag == "system_trust_veteran") {
        TrustRank::TrustedUser
    } else if tags.iter().any(|tag| tag == "system_trust_trusted") {
        TrustRank::KnownUser
    } else if tags.iter().any(|tag| tag == "system_trust_known") {
        TrustRank::User
    } else if tags.iter().any(|tag| tag == "system_trust_basic") {
        TrustRank::NewUser
    } else if tags.iter().any(|tag| tag == "system_trust_troll") {
        TrustRank::Visitor
    } else {
        TrustRank::Unknown
    }
}

fn troll_state(tags: &[String]) -> TrollState {
    if tags.iter().any(|tag| tag == "system_troll_confirmed") {
        TrollState::Confirmed
    } else if tags.iter().any(|tag| tag == "system_troll_probable") {
        TrollState::Probable
    } else {
        TrollState::None
    }
}

fn supporter_state(tags: &[String]) -> SupporterState {
    if tags.iter().any(|tag| tag == "system_supporter_early") {
        SupporterState::EarlySupporter
    } else if tags.iter().any(|tag| tag == "system_supporter") {
        SupporterState::Supporter
    } else {
        SupporterState::None
    }
}

fn admin_tag(tag: &str) -> Option<AdminTags> {
    match tag {
        "admin_moderator" => Some(AdminTags::IsModerator),
        "admin_show_mod_tag" => Some(AdminTags::ShowModTag),
        "admin_avatar_access" => Some(AdminTags::HasAdminAvatarAccess),
        "admin_world_access" => Some(AdminTags::HasAdminWorldAccess),
        "admin_canny_access" => Some(AdminTags::HasAdminCannyAccess),
        "admin_can_grant_licenses" => Some(AdminTags::CanGrantLicenses),
        "admin_lock_level" => Some(AdminTags::LockLevel),
        "admin_lock_tags" => Some(AdminTags::LockTags),
        "admin_official_thumbnail" => Some(AdminTags::HasOfficialThumbnail),
        _ => None,
    }
}

fn permission(tag: &str) -> Option<Permissions> {
    match tag {
        "system_world_access" => Some(Permissions::HasWorldAccess),
        "system_avatar_access" => Some(Permissions::HasAvatarAccess),
        "system_feedback_access" => Some(Permissions::HasFeedbackAccess),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_flat_current_user_response() {
        let user: User = serde_json::from_value(serde_json::json!({
            "id": "usr_123",
            "username": "legacy-name",
            "displayName": "DrapNard",
            "status": "active",
            "statusDescription": "working",
            "bio": "hello",
            "bioLinks": ["https://example.test"],
            "pronouns": "",
            "userIcon": "",
            "profilePicOverride": "",
            "profilePicOverrideThumbnail": "",
            "currentAvatarImageUrl": "https://example.test/avatar.png",
            "date_joined": "2020-01-01",
            "last_activity": "2026-07-08",
            "last_login": "2026-07-08",
            "last_mobile": null,
            "last_platform": "standalonewindows",
            "presence": {
                "platform": "web"
            },
            "isFriend": false,
            "friendKey": "friend-key",
            "tags": ["system_trust_veteran", "system_trust_trusted", "system_supporter", "language_fra"]
        }))
        .unwrap();

        assert_eq!(user.identity.id, "usr_123");
        assert_eq!(user.identity.display_name, "DrapNard");
        assert_eq!(user.presence.platform, "web");
        assert_eq!(user.tags.trust_rank, TrustRank::TrustedUser);
        assert_eq!(user.tags.supporter_state, SupporterState::Supporter);
        assert_eq!(user.tags.languages, vec!["fra"]);
        assert_eq!(
            user.profile.current_avatar_image_url,
            "https://example.test/avatar.png"
        );
    }

    #[test]
    fn deserializes_two_factor_required_response() {
        let user: User = serde_json::from_value(serde_json::json!({
            "requiresTwoFactorAuth": ["totp", "emailOtp"]
        }))
        .unwrap();

        assert_eq!(
            user.requires_two_factor_auth,
            Some(vec!["totp".to_string(), "emailOtp".to_string()])
        );
    }
}
