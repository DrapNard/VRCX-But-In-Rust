#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GroupPostVisibility {
    Public,
    Group,
    Role,
    Unknown,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupPosts {
    pub posts: Vec<GroupPost>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupPost {
    pub id: String,
    pub group_id: String,
    pub author_id: String,
    pub editor_id: Option<String>,
    pub role_ids: Vec<String>,
    pub title: String,
    pub text: String,
    pub image_id: Option<String>,
    pub image_url: Option<String>,
    pub visibility: GroupPostVisibility,
    pub created_at: String,
    pub updated_at: String,
}
