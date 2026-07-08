use crate::models::common::ImagePair;

use super::{PropReleaseStatus, PropUnityPackage};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropSummary {
    pub id: String,
    pub name: String,
    pub media: ImagePair,
}

pub type Props = crate::models::common::Paginated<Prop>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropPublishStatus {
    pub can_publish: bool,
    pub reason: Option<String>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub details: serde_json::Value,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Prop {
    pub id: String,
    pub name: String,
    pub description: String,
    pub author_id: String,
    pub author_name: String,
    pub image_url: String,
    pub thumbnail_image_url: String,
    pub release_status: PropReleaseStatus,
    pub max_count_per_user: u32,
    pub spawn_type: u32,
    pub world_placement_mask: u32,
    pub tags: Vec<String>,
    pub unity_package_url: String,
    pub unity_packages: Vec<PropUnityPackage>,
    pub created_at: String,
    pub updated_at: String,
}
