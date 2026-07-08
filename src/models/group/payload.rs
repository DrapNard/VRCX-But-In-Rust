#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupCreate {
    pub name: String,
    pub short_code: String,
    pub description: String,
    pub icon_id: Option<String>,
    pub banner_id: Option<String>,
    pub privacy: String,
    pub tags: Vec<String>,
    pub rules: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub icon_id: Option<String>,
    pub banner_id: Option<String>,
    pub privacy: Option<String>,
    pub tags: Vec<String>,
    pub rules: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupAnnouncementCreate {
    pub title: String,
    pub text: String,
    pub image_id: Option<String>,
    pub role_ids: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupGalleryCreate {
    pub name: String,
    pub description: String,
    pub members_only: bool,
    pub role_ids_to_view: Vec<String>,
    pub role_ids_to_submit: Vec<String>,
    pub role_ids_to_auto_approve: Vec<String>,
    pub role_ids_to_manage: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupGalleryUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub members_only: Option<bool>,
    pub role_ids_to_view: Vec<String>,
    pub role_ids_to_submit: Vec<String>,
    pub role_ids_to_auto_approve: Vec<String>,
    pub role_ids_to_manage: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupInviteCreate {
    pub user_id: String,
    pub role_ids: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupJoinRequestResponse {
    pub user_id: String,
    pub accept: bool,
    pub role_ids: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupMemberUpdate {
    pub visibility: Option<String>,
    pub manager_notes: Option<String>,
    pub role_ids: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupMemberRoleUpdate {
    pub user_id: String,
    pub role_id: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupPostCreate {
    pub title: String,
    pub text: String,
    pub image_id: Option<String>,
    pub role_ids: Vec<String>,
    pub visibility: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupPostUpdate {
    pub title: Option<String>,
    pub text: Option<String>,
    pub image_id: Option<String>,
    pub role_ids: Vec<String>,
    pub visibility: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupRoleCreate {
    pub name: String,
    pub description: String,
    pub permissions: Vec<String>,
    pub is_self_assignable: bool,
    pub requires_purchase: bool,
    pub requires_two_factor: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupRoleUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub permissions: Vec<String>,
    pub is_self_assignable: Option<bool>,
    pub requires_purchase: Option<bool>,
    pub requires_two_factor: Option<bool>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupRepresentationUpdate {
    pub group_id: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupTransferRequest {
    pub target_user_id: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupGalleryFileOrder {
    pub file_ids: Vec<String>,
}
