#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub bio: String,
    pub bio_links: Vec<String>,
    pub pronouns: String,
    pub user_icon: String,
    pub profile_pic_override: String,
    pub profile_pic_override_thumbnail: String,
}
