#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GroupAdminTag {
    AgeVerificationEnabled,
    HideMemberCount,
    FeaturedEventsEnabled,
    VrcEventGroupFairEnabled,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupTags {
    pub admin_tags: Vec<GroupAdminTag>,
    pub raw: Vec<String>,
}
