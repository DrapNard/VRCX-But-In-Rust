#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupAnnouncement {
    pub id: String,
    pub group_id: String,
    pub author_id: String,
    pub title: String,
    pub text: String,
    pub image_id: Option<String>,
    pub image_url: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
