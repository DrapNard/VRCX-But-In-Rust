#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImagePair {
    pub image_url: String,
    pub thumbnail_image_url: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OptionalImagePair {
    pub image_url: Option<String>,
    pub thumbnail_image_url: Option<String>,
}
