use super::ProductListing;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Economy {
    pub product_id: Option<String>,
    pub highest_price: Option<u32>,
    pub lowest_price: Option<u32>,
    pub published_listings: Vec<ProductListing>,
}
