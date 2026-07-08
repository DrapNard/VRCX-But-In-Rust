use super::PropReleaseStatus;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropCreate {
    pub name: String,
    pub description: String,
    pub image_id: Option<String>,
    pub max_count_per_user: u32,
    pub spawn_type: u32,
    pub world_placement_mask: u32,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub image_id: Option<String>,
    pub max_count_per_user: Option<u32>,
    pub spawn_type: Option<u32>,
    pub world_placement_mask: Option<u32>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropPublish {
    pub release_status: PropReleaseStatus,
}
