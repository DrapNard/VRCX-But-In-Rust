use super::ProductListing;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Store {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub store_id: String,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub seller_id: String,
    #[serde(default)]
    pub seller_display_name: String,
    #[serde(default)]
    pub store_type: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub shelves: Vec<StoreShelf>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreShelf {
    #[serde(default)]
    pub id: String,
    #[serde(default, alias = "shelfTitle")]
    pub display_name: String,
    #[serde(default, alias = "shelfDescription")]
    pub description: String,
    #[serde(default)]
    pub listings: Vec<ProductListing>,
}
