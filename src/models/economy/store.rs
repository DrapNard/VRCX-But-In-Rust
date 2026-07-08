use super::ProductListing;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Store {
    pub shelves: Vec<StoreShelf>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreShelf {
    pub id: String,
    pub display_name: String,
    pub description: String,
    pub listings: Vec<ProductListing>,
}
