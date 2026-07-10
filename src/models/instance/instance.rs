use super::{Capacity, Flags, Identity, Privacy, Region};
use crate::models::common::default_content_settings::DefaultContentSettings;
use crate::models::common::platform::Platform;
use crate::models::users::UserSummary;
use crate::models::world::WorldSummary;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceSummary {
    pub identity: Identity,
    pub platform: Platform,
    pub region: Region,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Instance {
    pub identity: Identity,
    pub platform: Platform,
    pub content_settings: Vec<DefaultContentSettings>,
    pub capacity: Capacity,
    pub privacy: Privacy,
    pub region: Region,
    pub world: WorldSummary,
    pub users: Vec<UserSummary>,
    pub flags: Vec<Flags>,
    pub owner: Option<UserSummary>,
}
