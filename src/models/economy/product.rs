pub type ProductListingList = crate::models::common::Paginated<ProductListing>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductListing {
    pub id: String,
    pub product_id: String,
    pub seller_id: String,
    pub display_name: String,
    pub description: String,
    pub image_id: Option<String>,
    pub image_url: Option<String>,
    pub price_tokens: u32,
    pub listing_type: String,
    pub active: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductPurchase {
    pub id: String,
    pub listing_id: String,
    pub buyer_id: String,
    pub seller_id: String,
    pub quantity: u32,
    pub price_tokens: u32,
    pub created_at: String,
}
