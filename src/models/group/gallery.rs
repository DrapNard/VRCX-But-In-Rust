#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupGallery {
    pub id: String,
    pub name: String,
    pub description: String,
    pub members_only: bool,
    pub role_ids_to_auto_approve: Vec<String>,
    pub role_ids_to_manage: Vec<String>,
    pub role_ids_to_submit: Vec<String>,
    pub role_ids_to_view: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupGalleryImage {
    pub id: String,
    pub group_id: String,
    pub gallery_id: String,
    pub file_id: String,
    pub image_url: String,
    pub approved: bool,
    pub approved_at: Option<String>,
    pub approved_by_user_id: Option<String>,
    pub submitted_by_user_id: String,
    pub created_at: String,
}
