#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GroupMemberStatus {
    Member,
    Requested,
    Invited,
    Banned,
    Unknown,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GroupMemberVisibility {
    Visible,
    Hidden,
    Unknown,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupUserSummary {
    pub id: String,
    pub display_name: String,
    pub icon_url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub profile_pic_override: Option<String>,
    pub current_avatar_thumbnail_image_url: Option<String>,
    pub current_avatar_tags: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupMember {
    pub id: String,
    pub group_id: String,
    pub user_id: String,
    pub user: Option<GroupUserSummary>,
    pub role_ids: Vec<String>,
    pub m_role_ids: Vec<String>,
    pub permissions: Vec<String>,
    pub membership_status: GroupMemberStatus,
    pub visibility: GroupMemberVisibility,
    pub manager_notes: Option<String>,
    pub accepted_by_id: Option<String>,
    pub accepted_by_display_name: Option<String>,
    pub has_2fa: Option<bool>,
    pub has_joined_from_purchase: bool,
    pub is_representing: bool,
    pub is_subscribed_to_announcements: bool,
    pub is_subscribed_to_event_announcements: bool,
    pub banned_at: Option<String>,
    pub created_at: String,
    pub joined_at: Option<String>,
    pub last_post_read_at: Option<String>,
}
