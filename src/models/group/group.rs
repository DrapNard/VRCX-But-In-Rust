use super::{GroupGallery, GroupMember, GroupRole, GroupTags};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupSummary {
    pub id: String,
    pub name: String,
    pub short_code: String,
    pub discriminator: String,
    pub description: String,
    pub icon_url: Option<String>,
    pub banner_url: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupSearchResult {
    pub id: String,
    pub name: String,
    pub short_code: String,
    pub discriminator: String,
    pub description: String,
    pub owner_id: String,
    pub icon_id: Option<String>,
    pub icon_url: Option<String>,
    pub banner_id: Option<String>,
    pub banner_url: Option<String>,
    pub galleries: Vec<GroupGallery>,
    pub member_count: u32,
    pub membership_status: Option<String>,
    pub rules: Option<String>,
    pub tags: GroupTags,
    pub is_searchable: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Group {
    pub id: String,
    pub name: String,
    pub short_code: String,
    pub discriminator: String,
    pub description: String,
    pub owner_id: String,
    pub transfer_target_id: Option<String>,
    pub icon_id: Option<String>,
    pub icon_url: Option<String>,
    pub banner_id: Option<String>,
    pub banner_url: Option<String>,
    pub badges: Vec<String>,
    pub galleries: Vec<GroupGallery>,
    pub roles: Vec<GroupRole>,
    pub my_member: Option<GroupMember>,
    pub join_state: String,
    pub membership_status: Option<String>,
    pub privacy: String,
    pub rules: Option<String>,
    pub tags: GroupTags,
    pub languages: Vec<String>,
    pub links: Vec<String>,
    pub member_count: u32,
    pub online_member_count: Option<u32>,
    pub member_count_synced_at: Option<String>,
    pub age_verification_beta_code: Option<String>,
    pub age_verification_beta_slots: Option<u32>,
    pub age_verification_slots_available: bool,
    pub allow_group_join_prompt: bool,
    pub is_verified: bool,
    pub created_at: String,
    pub updated_at: String,
    pub last_post_created_at: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_group_when_roles_and_optional_fields_are_absent() {
        let group: Group = serde_json::from_value(serde_json::json!({
            "id": "grp_123",
            "name": "Example",
            "shortCode": "EX",
            "description": "",
            "ownerId": "usr_123",
            "iconUrl": null,
            "bannerUrl": null,
            "badges": [],
            "galleries": [],
            "joinState": "invite",
            "membershipStatus": "inactive",
            "privacy": "default",
            "tags": [],
            "languages": [],
            "links": [],
            "memberCount": 0,
            "isVerified": false
        }))
        .unwrap();

        assert!(group.roles.is_empty());
        assert_eq!(group.id, "grp_123");
    }
}
