use crate::models::world::WorldSummary;

pub type JamSubmissions = crate::models::common::Paginated<JamSubmission>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JamSubmission {
    pub id: String,
    pub jam_id: String,
    pub world_id: String,
    pub world: Option<WorldSummary>,
    pub author_id: String,
    pub author_name: String,
    pub title: String,
    pub description: String,
    pub image_url: Option<String>,
    pub thumbnail_image_url: Option<String>,
    pub tags: Vec<String>,
    pub score: Option<u32>,
    pub rank: Option<u32>,
    pub submitted_at: String,
    pub updated_at: String,
}
