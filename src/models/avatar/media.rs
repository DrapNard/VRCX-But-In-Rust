use crate::models::common::ImagePair;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Media {
    pub images: ImagePair,
}
