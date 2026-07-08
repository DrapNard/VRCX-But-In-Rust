use crate::models::instance::InstanceSummary;

use super::{
    Tags,
    Capacity,
    Content,
    Identity,
    Media,
    Publication,
    Stats,
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldSummary {
    pub identifier: Identity,
    pub media: Media,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct World {
    pub identifier: Identity,
    pub content: Content,
    pub media: Media,
    pub publications: Publication,
    pub stats: Stats,
    pub capacity: Capacity,
    pub tags: Tags,
    pub instance: Option<InstanceSummary>,
}
