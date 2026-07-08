#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Badges {
    pub badge_id: String,
    pub badge_name: String,
    pub badge_description: String,
    pub badge_image_url: String,
    pub assigned_at: String,
    pub updated_at: String,
    pub hidden: bool,
    pub showcased: bool,
}
