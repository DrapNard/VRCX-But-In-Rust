#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductListing {
    pub description: String,
    pub display_name: String,
    pub image_id: String,
    pub listing_id: String,
    pub listing_type: String,
    pub price_tokens: u32,
}
