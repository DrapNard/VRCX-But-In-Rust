use super::{Badges, Identity, Metadata, Presence, Profile, Social, Tags};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSummary {
    pub identity: Identity,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub profile: Profile,
    pub identity: Identity,
    pub metadata: Metadata,
    pub presence: Presence,
    pub social: Social,
    pub badges: Vec<Badges>,
    pub tags: Tags,
}
