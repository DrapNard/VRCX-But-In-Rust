#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Media {
    pub image_url: String,
    pub thumbnail_image_url: String,
    pub preview_youtube_id: String,
    pub url_lists: Vec<String>,
}
